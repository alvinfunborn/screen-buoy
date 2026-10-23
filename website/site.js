const demo = document.querySelector('#demo');
const toggle = document.querySelector('#demo-toggle');
const status = document.querySelector('#demo-status');
const cards = [...document.querySelectorAll('[data-hint]')];
let active = false;

function showHints(show) {
  active = show;
  toggle.setAttribute('aria-pressed', String(show));
  toggle.innerHTML = `${show ? '隐藏 Hint' : '显示 Hint'} <span aria-hidden="true">↵</span>`;
  cards.forEach(card => {
    card.querySelector('.hint-tag').hidden = !show;
    card.classList.remove('selected');
  });
  status.textContent = show ? '试着按 F、J 或 K。Esc 可退出。' : '先唤出提示，再按一个字母。';
}

function selectCard(card) {
  showHints(false);
  card.classList.add('selected');
  status.textContent = `已点击「${card.dataset.name}」。再显示 Hint，试试另一个。`;
}

toggle.addEventListener('click', () => {
  showHints(!active);
  if (active) demo.focus({ preventScroll: true });
});
cards.forEach(card => card.addEventListener('click', () => selectCard(card)));
document.querySelector('#try-demo').addEventListener('click', () => {
  showHints(true);
  demo.focus({ preventScroll: true });
});
// Only handle keys while the visitor is interacting with the demo.
// The site's navigation and browser/system shortcuts remain available.
demo.addEventListener('keydown', event => {
  if (event.altKey || event.metaKey || event.ctrlKey || event.repeat) return;
  if (event.key === 'Escape' && active) {
    event.preventDefault();
    showHints(false);
    toggle.focus({ preventScroll: true });
  } else if (active) {
    // Physical letter keys keep the demo usable with a Chinese IME enabled.
    const letter = /^Key[A-Z]$/.test(event.code) ? event.code.slice(3).toLowerCase() : event.key.toLowerCase();
    const card = cards.find(item => item.dataset.hint === letter);
    if (card) {
      event.preventDefault();
      selectCard(card);
    }
  }
});
