"""Exercise the built WASM game in Firefox (requires selenium and geckodriver)."""
import functools
import http.server
import os
import re
from pathlib import Path
import threading

from selenium import webdriver
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.support.ui import WebDriverWait

ROOT = Path(__file__).resolve().parents[1]
handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(ROOT / 'demo'))
server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), handler)
threading.Thread(target=server.serve_forever, daemon=True).start()
options = webdriver.FirefoxOptions()
options.add_argument('-headless')
options.set_preference('ui.prefersReducedMotion', 0)
driver_path = os.environ.get('GECKODRIVER')
service = Service(executable_path=driver_path) if driver_path else Service()
driver = webdriver.Firefox(options=options, service=service)
try:
    driver.set_window_size(1280, 900)
    driver.get(f'http://127.0.0.1:{server.server_port}/')
    wait = WebDriverWait(driver, 30)
    wait.until(lambda d: d.find_elements('css selector', '.card-button'))
    # Accelerate subsequent game timers, without changing the Rust game state.
    driver.execute_script('''
        window.errors = [];
        window.addEventListener('error', e => window.errors.push(e.message));
        const original = window.setTimeout;
        window.setTimeout = (fn, delay, ...args) => original(fn, Math.min(delay, 8), ...args);
        window.autoplay = window.setInterval(() => document.querySelector('.card-button')?.click(), 15);
    ''')
    for mode in ['CHALLENGER', 'RANDOM']:
        driver.execute_script('''
            [...document.querySelectorAll('.terminal-button')].find(b => b.textContent === arguments[0]).click();
        ''', mode)
        wait.until(lambda d: d.find_elements('css selector', '.victory-result'))
        result = driver.find_element('css selector', '.victory-result').text
        sparks = driver.find_elements('css selector', '.firework-burst span')
        assert len(sparks) == (0 if result.startswith('Draw') else 36), result
        scores = [int(value) for value in re.findall(r'\d+', result)]
        assert sum(scores) == 120, result
        assert not driver.find_elements('css selector', '.card-button')
        print(mode, result.replace('\n', ' / '))
    driver.save_screenshot('/tmp/briscola-result.png')
    driver.execute_script("document.body.setAttribute('data-production-theme', 'white'); [...document.querySelectorAll('.terminal-button')].find(b => b.textContent === 'BIGGER VIEW').click()")
    assert driver.find_elements('css selector', '.theme-white.fill-screen .victory-result')
    driver.set_window_size(390, 844)
    assert driver.execute_script('return document.documentElement.scrollWidth <= window.innerWidth')
    driver.save_screenshot('/tmp/briscola-result-mobile.png')
    driver.execute_script("clearInterval(window.autoplay); [...document.querySelectorAll('.terminal-button')].find(b => b.textContent === 'NEW').click()")
    assert not driver.find_elements('css selector', '.victory-layer')
    wait.until(lambda d: d.find_elements('css selector', '.card-button'))
    assert driver.execute_script('return window.errors') == []
    print('PASS: complete games, result effects, themes, mobile width, restart, no browser errors')
finally:
    driver.quit()
    server.shutdown()
