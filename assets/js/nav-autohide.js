/*
  The storefront's top bars (navbar + category row) slide away while the
  shopper scrolls down and come straight back on any scroll up, so the
  page gets the whole screen while reading and the nav is one flick away.
  Started from navbar.js, so it runs on every storefront page.

  State lives in two classes on <html>, styled in common.css/shopping.css:
    nav-hidden     — the bars are slid up out of view
    nav-past-hero  — home page only: the whole hero has scrolled out of
                     view, so the bar that was drawn inside it comes back
                     (on scroll-up) as the ordinary white bar, fixed to
                     the top. Anywhere over the hero there is no white
                     bar at all — only the one inside the photo.
    nav-snap       — held for a frame at that hand-over, so the bar
                     changes places without a visible slide
*/
const root = document.documentElement;
const navbar = document.querySelector('.navbar');

// Movement smaller than this is ignored, so a trackpad's jitter or the
// tail of a smooth-scroll glide doesn't flicker the bar.
const THRESHOLD = 6;

function barsHeight() {
  const sub = document.querySelector('.subnav');
  return (navbar ? navbar.offsetHeight : 0) + (sub ? sub.offsetHeight : 0);
}

// Never hide the bar out from under someone using it: a focused search,
// an open "Deliver to" menu or account menu, or the pointer resting on it.
function navInUse() {
  if (!navbar) return false;
  return navbar.matches(':hover, :focus-within') ||
    !!navbar.querySelector('[aria-expanded="true"]');
}

function start() {
  if (!navbar) return;
  const hero = document.querySelector('.has-hero-overlay .hero');
  let lastY = window.scrollY;
  let ticking = false;

  const update = () => {
    ticking = false;
    const y = Math.max(0, window.scrollY);

    const delta = y - lastY;

    if (hero) {
      // Past the hero's foot, not just past the bar inside it: scrolling
      // back up over the photo must not pull the white bar down on it.
      const pastHero = y > hero.offsetTop + hero.offsetHeight;
      if (pastHero !== root.classList.contains('nav-past-hero')) {
        root.classList.add('nav-snap');
        root.classList.toggle('nav-past-hero', pastHero);
        // Leaving on the way down, the white bar starts tucked away and
        // comes back on the next scroll up, instead of appearing and
        // sliding off over the page.
        if (pastHero && delta >= 0) root.classList.add('nav-hidden');
        requestAnimationFrame(() => requestAnimationFrame(() => root.classList.remove('nav-snap')));
      }
    }

    if (Math.abs(delta) < THRESHOLD) return;

    if (y <= barsHeight()) {
      root.classList.remove('nav-hidden');          // at the top: always shown
    } else if (delta > 0 && !navInUse()) {
      root.classList.add('nav-hidden');             // scrolling down
    } else if (delta < 0) {
      root.classList.remove('nav-hidden');          // scrolling up
    }
    lastY = y;
  };

  window.addEventListener('scroll', () => {
    if (!ticking) { ticking = true; requestAnimationFrame(update); }
  }, { passive: true });

  // Tabbing into the bar while it's hidden brings it back.
  navbar.addEventListener('focusin', () => root.classList.remove('nav-hidden'));

  update();
}

start();
