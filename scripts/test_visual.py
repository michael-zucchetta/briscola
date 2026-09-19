"""Deterministic Firefox E2E screenshots of the real WASM game."""
import argparse
import functools
import http.server
import io
import json
import os
from pathlib import Path
import threading

from PIL import Image, ImageChops
from selenium import webdriver
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.support.ui import WebDriverWait

ROOT = Path(__file__).resolve().parents[1]
BOOTSTRAP = '''<script>
window.testErrors = [];
addEventListener('error', e => testErrors.push(e.message));
addEventListener('unhandledrejection', e => testErrors.push(String(e.reason)));
let seed = 20260915;
crypto.getRandomValues = array => {
    const bytes = new Uint8Array(array.buffer, array.byteOffset, array.byteLength);
    for (let i = 0; i < bytes.length; i++) {
        seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5;
        bytes[i] = seed & 255;
    }
    return array;
};
window.gameTimers = [];
window.setTimeout = (fn, delay, ...args) => gameTimers.push(() => fn(...args));
window.stepGame = () => { const fn = gameTimers.shift(); if (fn) fn(); return !!fn; };
</script>'''


class Handler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        if self.path == '/':
            page = (ROOT / 'demo/index.html').read_text()
            page = page.replace('<head>', '<head>' + BOOTSTRAP)
            page = page.replace('data-production-theme="white"', 'data-production-theme="terminal"')
            body = page.encode()
            self.send_response(200)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        else:
            super().do_GET()


def compare(actual, baseline, diff_path):
    """Exact pixels: deterministic state/animation, fixed browser environment."""
    expected = Image.open(baseline).convert('RGB')
    if actual.size != expected.size:
        return f'size changed: {expected.size} -> {actual.size}'
    diff = ImageChops.difference(actual, expected)
    if diff.getbbox():
        diff.save(diff_path)
        return f'pixels differ; see {diff_path}'
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--update-baselines', action='store_true')
    parser.add_argument('--baseline-dir', type=Path, default=ROOT / 'tests/visual/baselines')
    parser.add_argument('--output-dir', type=Path, default=ROOT / 'test-results/visual')
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    if args.update_baselines:
        args.baseline_dir.mkdir(parents=True, exist_ok=True)
    server = http.server.ThreadingHTTPServer(
    ('127.0.0.1', 0), functools.partial(Handler, directory=str(ROOT / 'demo')))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    options = webdriver.FirefoxOptions()
    options.add_argument('-headless')
    options.set_preference('layout.css.devPixelsPerPx', '1.0')
    options.set_preference('ui.prefersReducedMotion', 0)
    path = os.environ.get('GECKODRIVER')
    service = Service(executable_path=path) if path else Service()
    driver = None
    failures = []
    try:
        driver = webdriver.Firefox(options=options, service=service)
        metadata = {'browser': driver.capabilities['browserVersion'], 'platform': driver.capabilities['platformName']}
        (args.output_dir / 'environment.json').write_text(json.dumps(metadata, indent=2) + '\n')

        def viewport(width, height):
            driver.set_window_size(width, height)
            extra = driver.execute_script('return [outerWidth-innerWidth, outerHeight-innerHeight]')
            driver.set_window_size(width + extra[0], height + extra[1])
            assert driver.execute_script('return [innerWidth,innerHeight]') == [width, height]

        def step_until(selector):
            for _ in range(250):
                if driver.find_elements('css selector', selector):
                    return
                assert driver.execute_script('return stepGame()'), f'No timer before {selector}'
            raise AssertionError(f'Did not reach {selector}')

        def control(label):
            driver.find_element('xpath', f'//button[text()="{label}"]').click()

        def snapshot(name):
            assert driver.execute_script('return testErrors') == []
            assert driver.execute_script('return document.documentElement.scrollWidth <= innerWidth')
            # Blur clicks and seek every animation to the same point in its timeline.
            driver.execute_script('''
                document.activeElement.blur();
                for (const animation of document.getAnimations()) {
                    animation.pause(); animation.currentTime = 700;
                }
            ''')
            driver.execute_async_script('''
                const done = arguments[0];
                document.fonts.ready.then(() => requestAnimationFrame(() => requestAnimationFrame(done)));
            ''')
            actual = Image.open(io.BytesIO(driver.get_screenshot_as_png())).convert('RGB')
            actual.save(args.output_dir / f'{name}.png')
            baseline = args.baseline_dir / f'{name}.png'
            if args.update_baselines:
                actual.save(baseline)
            elif not baseline.exists():
                failures.append(f'{name}: missing baseline {baseline}')
            else:
                error = compare(actual, baseline, args.output_dir / f'{name}.diff.png')
                if error:
                    failures.append(f'{name}: {error}')
            print(f'Captured {name}', flush=True)

        viewport(1280, 900)
        driver.get(f'http://127.0.0.1:{server.server_port}/')
        WebDriverWait(driver, 30).until(lambda d: d.find_elements('css selector', '.dashboard'))
        step_until('.card-button')
        assert not driver.find_elements('css selector', '.briscola-app.fill-screen')
        snapshot('terminal-dealt')
        # Check normal mobile view too: previous snapshots only covered BIGGER VIEW,
        # which masked a collapsed dashboard caused by min-height: 100%.
        # A same-origin iframe provides a real 390px CSS viewport even when
        # Firefox's desktop window manager enforces a wider minimum window.
        for width in (390, 520):
            driver.execute_script("""
                const frame = document.createElement('iframe');
                frame.id = 'mobile-test';
                frame.src = '/';
                frame.style.cssText = `position:fixed;inset:0;border:0;width:${arguments[0]}px;height:844px;z-index:2147483647`;
                document.body.append(frame);
            """, width)
            driver.switch_to.frame('mobile-test')
            WebDriverWait(driver, 30).until(lambda d: d.find_elements('css selector', '.dashboard'))
            step_until('.card-button')
            assert driver.find_elements('css selector', '.briscola-app.fill-screen')
            assert driver.find_elements('xpath', '//button[text()="BIGGER VIEW" and contains(@class,"active")]')
            control('NEW')
            assert driver.find_elements('css selector', '.briscola-app.fill-screen')
            step_until('.card-button')
            normal_card_widths = {}
            for expanded in (False, True):
                control('BIGGER VIEW')
                if not expanded:
                    control('NEW')
                    assert not driver.find_elements('css selector', '.briscola-app.fill-screen')
                    step_until('.card-button')
                for height in (844, 640):
                    driver.switch_to.default_content()
                    driver.execute_script("document.getElementById('mobile-test').style.height = arguments[0] + 'px'", height)
                    driver.switch_to.frame('mobile-test')
                    layout = driver.execute_script('''
                        const rect = selector => document.querySelector(selector).getBoundingClientRect();
                        const dashboard = rect('.dashboard');
                        const footer = rect('.footer-bar');
                        const status = rect('.trick-status');
                        const rail = rect('.header-rail');
                        const score = rect('.trick-status + .header-pill');
                        return {
                            cardWidth: rect('.card-button').width,
                            boardOverflow: document.querySelector('.table-board').scrollHeight >
                                document.querySelector('.table-board').clientHeight + 1,
                            panelsFit: [...document.querySelectorAll('.stack-panel')].every(panel =>
                                panel.scrollWidth <= panel.clientWidth + 1),
                            viewport: innerHeight,
                            dashboardHeight: dashboard.height,
                            footerBottom: footer.bottom,
                            statusWidth: status.width,
                            railWidth: rail.width,
                            statusBottom: status.bottom,
                            scoreTop: score.top,
                            overflow: document.documentElement.scrollWidth > innerWidth,
                            expanded: document.querySelector('.briscola-app').classList.contains('fill-screen')
                        };
                    ''')
                    context = f'mobile width={width}, expanded={expanded}, height={height}: {layout}'
                    assert layout['expanded'] == expanded, context
                    assert abs(layout['dashboardHeight'] - layout['viewport']) <= 1, context
                    assert layout['viewport'] - 12 <= layout['footerBottom'] <= layout['viewport'], context
                    assert abs(layout['statusWidth'] - layout['railWidth']) <= 1, context
                    assert layout['scoreTop'] >= layout['statusBottom'], context
                    assert not layout['overflow'], context
                    assert not layout['boardOverflow'], context
                    assert layout['panelsFit'], context
                    if expanded:
                        assert layout['cardWidth'] >= normal_card_widths[height] * 1.4, context
                    else:
                        normal_card_widths[height] = layout['cardWidth']
            driver.save_screenshot(args.output_dir / f'mobile-bigger-{width}.png')
            driver.switch_to.default_content()
            driver.execute_script("document.getElementById('mobile-test').remove()")
        driver.find_element('css selector', '.card-button').click()
        for _ in range(10):
            if len(driver.find_elements('css selector', '.trick-slot .card-shell')) == 2:
                break
            assert driver.execute_script('return stepGame()')
        assert len(driver.find_elements('css selector', '.trick-slot .card-shell')) == 2
        snapshot('terminal-trick')
        moves = 1
        for _ in range(250):
            if driver.find_elements('css selector', '.victory-result'):
                break
            cards = driver.find_elements('css selector', '.card-button')
            if cards:
                cards[0].click()
                moves += 1
            else:
                assert driver.execute_script('return stepGame()')
        assert moves == 20
        assert driver.find_elements('css selector', '.victory-result')
        assert len(driver.find_elements('css selector', '.firework-burst span')) == 36
        assert not driver.find_elements('css selector', '.card-button')
        snapshot('terminal-result')
        driver.execute_script('document.body.dataset.productionTheme = "white"')
        control('BIGGER VIEW')
        viewport(520, 844)
        snapshot('white-narrow-result')
        control('NEW')
        assert not driver.find_elements('css selector', '.victory-layer')
        step_until('.card-button')
        snapshot('white-narrow-restarted')
        if args.update_baselines:
            (args.baseline_dir / 'environment.json').write_text(json.dumps(metadata, indent=2) + '\n')
        if failures:
            raise AssertionError('\n'.join(failures))
        print('PASS: E2E assertions, mobile height/status regression checks and five visual snapshots' + (' (baselines updated)' if args.update_baselines else ''))
    finally:
        if driver:
            driver.quit()
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    main()
