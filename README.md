# Briscola
[Briscola](https://en.wikipedia.org/wiki/Briscola) is an italian card game.
[Play the live demo](https://michael-zucchetta.github.io/briscola/)

This project is a Rust Briscola implementation with a WebAssembly browser UI. The browser build is designed to mount inside a small static page, so it can be served directly from `demo/` or embedded in a portfolio project page.


## AI ipmplementation

The current AI has two modes:

- `Random`: picks any legal card from the AI hand.
- `Challenger`: a deterministic heuristic player that tries to make sensible low-risk Briscola moves.

The Challenger AI is built in `src/ai.rs` around a small visible-state contract:

- the current briscola suit
- the current lead card, if the AI is following
- the cards in the AI hand

It does not inspect the opponent hand, count all previously played cards, search future tricks, or run a minimax/Monte Carlo simulation. It only plays from information that is visible to a normal player at the table plus its own hand.

When Challenger leads a trick, it prefers the cheapest low-value non-trump card available. This avoids throwing away points or spending briscola cards before there is a reason to do so.

When Challenger follows a trick, it checks whether it can win the lead card using the shared Briscola rules in `src/rules.rs`. If the lead card is worth points, Challenger tries to win it with the cheapest winning card. If the lead card has no points, or if Challenger cannot win, it discards the cheapest card available.

Card cost is based on two things:

- point value, so aces, threes, kings, cavalli, and jacks are treated as expensive
- trump status, so briscola cards are preserved unless spending one wins useful points

This makes Challenger stronger than a random legal player, but still intentionally lightweight. It is a fast browser-safe heuristic AI, not a full perfect-information solver.

## Build

```sh
make build
```

The build writes the WebAssembly package and browser demo into `demo/`.

### GitHub Pages

Run `make github-pages` to build the white production theme and copy the website
from `demo/` into `docs/`, including `.nojekyll`. Then commit and push the output:

```sh
git add docs
git commit -m "Publish Briscola static website"
git push
```

In GitHub **Settings → Pages**, select **Deploy from a branch**, your pushed
branch, and **/docs**. Run the target and commit the updated output whenever
you want to publish changes. The target prepares files locally; it does not
commit or push them.

## Test

```sh
cargo test
```

## Game results and browser verification

After `make build`, run `python3 scripts/verify_firefox.py` for a Firefox smoke
test covering full games in both AI modes, final results, restart, and layout.
It requires Python Selenium, Firefox, and geckodriver (optionally set its path
with `GECKODRIVER`). The test accelerates browser timers to finish games quickly.
