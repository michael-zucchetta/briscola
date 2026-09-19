// Delegated listeners survive the Rust renderer replacing the dashboard.
export function installCardZoom(app) {
    const selector = '.card-button, .card-shell:not(.card-back)';
    const preview = document.createElement('div');
    preview.className = 'card-zoom';
    preview.setAttribute('aria-hidden', 'true');
    const picture = document.createElement('img');
    picture.alt = '';
    const hint = document.createElement('div');
    hint.className = 'card-zoom-hint';
    preview.append(picture, hint);
    let source = null;
    let pinned = false;
    let gesture = null;
    let tap = null;
    let pointerType = '';

    function cardAt(target) {
        const card = target instanceof Element ? target.closest(selector) : null;
        return card && app.contains(card) ? card : null;
    }
    function dismiss() {
        preview.remove();
        source = null;
        pinned = false;
        tap = null;
    }
    function show(card, pin = false) {
        const svg = card.querySelector('svg');
        if (!svg) return;
        source = card;
        pinned = pin;
        // An isolated image avoids duplicate SVG IDs and pattern references.
        picture.src = 'data:image/svg+xml;charset=utf-8,' +
            encodeURIComponent(new XMLSerializer().serializeToString(svg));
        hint.textContent = pin
            ? (card.matches('.card-button') ? 'Tap card again to play' : 'Tap outside to close')
            : '';
        hint.hidden = !pin;
        const rect = card.getBoundingClientRect();
        const viewport = window.visualViewport;
        const left = viewport?.offsetLeft || 0;
        const top = viewport?.offsetTop || 0;
        const width = viewport?.width || innerWidth;
        const height = viewport?.height || innerHeight;
        const ratio = svg.viewBox.baseVal.width / svg.viewBox.baseVal.height || 0.5;
        const captionHeight = pin ? 48 : 0;
        const imageWidth = Math.max(1, Math.min(rect.width * 2.2, 240,
            width - 24, (height - 24 - captionHeight) * ratio));
        const totalHeight = imageWidth / ratio + captionHeight;
        preview.style.width = `${imageWidth}px`;
        picture.style.height = `${imageWidth / ratio}px`;
        preview.style.left = `${Math.max(left + 12, Math.min(
            rect.left + rect.width / 2 - imageWidth / 2, left + width - imageWidth - 12))}px`;
        preview.style.top = `${Math.max(top + 12, Math.min(
            rect.bottom - totalHeight, top + height - totalHeight - 12))}px`;
        app.append(preview);
    }
    app.addEventListener('pointerover', event => {
        if (event.pointerType !== 'mouse') return;
        const card = cardAt(event.target);
        if (card && card !== cardAt(event.relatedTarget)) show(card);
    });
    app.addEventListener('pointerout', event => {
        if (event.pointerType === 'mouse' && !pinned && source &&
            cardAt(event.relatedTarget) !== source) dismiss();
    });
    document.addEventListener('pointerdown', event => {
        pointerType = event.pointerType;
        tap = null;
        gesture = event.isPrimary && event.pointerType !== 'mouse'
            ? {card: cardAt(event.target), x: event.clientX, y: event.clientY, id: event.pointerId}
            : null;
    }, true);
    document.addEventListener('pointermove', event => {
        if (gesture && (Math.hypot(event.clientX - gesture.x, event.clientY - gesture.y) > 10)) {
            gesture = null;
        }
    }, true);
    document.addEventListener('pointercancel', () => { gesture = null; tap = null; }, true);
    document.addEventListener('pointerup', event => {
        if (gesture && event.pointerId === gesture.id &&
            cardAt(event.target) === gesture.card) tap = gesture.card;
        gesture = null;
    }, true);
    document.addEventListener('click', event => {
        const card = cardAt(event.target);
        const isTap = event.detail !== 0 && card && tap === card && pointerType !== 'mouse';
        tap = null;
        if (isTap) {
            if (pinned && source === card) {
                dismiss(); // The original Rust click handler may now play it.
            } else {
                event.preventDefault();
                event.stopImmediatePropagation();
                show(card, true);
            }
        } else {
            dismiss();
        }
    }, true);
    app.addEventListener('focusin', event => {
        const card = cardAt(event.target);
        if (card && pointerType !== 'touch' && pointerType !== 'pen') show(card);
    });
    app.addEventListener('focusout', () => { if (!pinned) dismiss(); });
    document.addEventListener('keydown', event => {
        pointerType = '';
        if (event.key === 'Escape') dismiss();
    });
    const reset = () => { gesture = null; dismiss(); };
    window.addEventListener('resize', reset);
    document.addEventListener('scroll', reset, true);
    window.visualViewport?.addEventListener('resize', reset);
    window.visualViewport?.addEventListener('scroll', reset);
    function describeCards() {
        for (const card of app.querySelectorAll(selector)) {
            card.setAttribute('aria-describedby', 'card-zoom-help');
            if (!card.matches('button')) {
                card.setAttribute('tabindex', '0');
                card.setAttribute('aria-label', 'Enlarge card');
            }
        }
    }
    describeCards();
    new MutationObserver(() => {
        describeCards();
        if (source && !app.contains(source)) dismiss();
    }).observe(app, {childList: true, subtree: true});
}
