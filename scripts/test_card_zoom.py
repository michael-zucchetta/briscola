"""Exercise card previews against the real WASM app in Firefox."""
import functools
import http.server
import os
import threading
from selenium import webdriver
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.support.ui import WebDriverWait
from test_visual import Handler, ROOT

server = http.server.ThreadingHTTPServer(
    ('127.0.0.1', 0), functools.partial(Handler, directory=str(ROOT / 'demo')))
threading.Thread(target=server.serve_forever, daemon=True).start()
options = webdriver.FirefoxOptions()
options.add_argument('-headless')
driver = webdriver.Firefox(options=options, service=Service(executable_path=os.environ.get('GECKODRIVER')))
try:
    driver.set_window_size(1280, 900)
    driver.get(f'http://127.0.0.1:{server.server_port}/')
    WebDriverWait(driver, 30).until(lambda d: d.find_elements('css selector', '.dashboard'))
    def cards():
        return driver.find_elements('css selector', '.card-button')
    def preview():
        return driver.find_elements('css selector', '.card-zoom')
    def ready():
        for _ in range(30):
            if cards():
                return
            assert driver.execute_script('return stepGame()')
        raise AssertionError('No playable cards')
    def tap(card, cancel=False, move=False):
        # Touch pointer sequences test capture ordering before the Rust click handler.
        driver.execute_script('''
            const card = arguments[0], cancel = arguments[1], move = arguments[2];
            const options = {bubbles:true, pointerType:'touch', pointerId:3,
                             isPrimary:true, clientX:30, clientY:30};
            card.dispatchEvent(new PointerEvent('pointerdown', options));
            if (move) card.dispatchEvent(new PointerEvent('pointermove', {...options, clientY:80}));
            card.dispatchEvent(new PointerEvent(cancel ? 'pointercancel' : 'pointerup', options));
            if (!cancel && !move) card.dispatchEvent(new PointerEvent('click', {...options, detail:1}));
        ''', card, cancel, move)
    ready()
    first = cards()[0]
    before = first.rect
    ActionChains(driver).move_to_element(first).perform()
    assert preview()
    assert preview()[0].rect['height'] > before['height']
    assert driver.execute_script('return getComputedStyle(document.querySelector(".card-zoom")).pointerEvents') == 'none'
    WebDriverWait(driver, 10).until(lambda d: d.execute_script('return document.querySelector(".card-zoom img").naturalWidth > 0'))
    ActionChains(driver).send_keys(Keys.ESCAPE).perform()
    assert not preview()
    ActionChains(driver).move_to_element(driver.find_element('css selector', '.table-title')).perform()
    driver.execute_script('arguments[0].focus()', first)
    assert preview()
    ActionChains(driver).send_keys(Keys.ENTER).perform()
    assert not preview() and len(cards()) == 0
    ready()
    first, second = cards()[:2]
    tap(first, cancel=True)
    assert not preview()
    tap(first, move=True)
    assert not preview()
    tap(first)
    assert preview() and len(cards()) == 3
    assert 'again to play' in preview()[0].text
    tap(second)
    assert preview() and len(cards()) == 3
    tap(second)
    assert not preview() and len(cards()) == 0
    ready()
    # Mouse retains a single click to play.
    cards()[0].click()
    assert not preview() and len(cards()) == 0
    ready()
    tap(cards()[0])
    driver.find_element('css selector', '.table-title').click()
    assert not preview()
    back = driver.find_element('css selector', '.card-back')
    tap(back)
    assert not preview()
    for theme in ('terminal', 'white'):
        driver.execute_script('document.body.dataset.productionTheme = arguments[0]', theme)
        driver.find_element('xpath', '//button[text()="BIGGER VIEW"]').click()
        driver.set_window_size(390, 700)
        first = cards()[0]
        tap(first)
        assert preview()
        assert driver.execute_script('''
            const r = document.querySelector('.card-zoom').getBoundingClientRect();
            return r.left >= 0 && r.top >= 0 && r.right <= innerWidth && r.bottom <= innerHeight;
        ''')
        driver.save_screenshot(str(ROOT / f'test-results/zoom-{theme}.png'))
        driver.set_window_size(420, 740)
        assert not preview()
    tap(cards()[0])
    # Trigger a real redraw while a preview is pinned.
    driver.execute_script("document.querySelector('.terminal-button').click()")
    assert not preview()
    ready()
    tap(cards()[0])
    assert preview()
    assert driver.execute_script('return testErrors') == []
    print('PASS: hover, keyboard, mouse, touch, cancellation, switching, dismissal, hidden cards, redraw, viewport and themes')
finally:
    driver.quit()
    server.shutdown()
    server.server_close()
