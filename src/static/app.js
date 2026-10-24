/* Ayanami_Rei · 交互层（方向 B）
   只处理表现层状态：批量模式、勾选计数、底部操作坞、筛选条件编辑器、密度切换。
   所有写操作仍走各自的 <form> POST + CSRF hidden 字段，本文件不构造任何请求。
*/
(() => {
  'use strict';

  /* ─────────── 筛选条件编辑器 ─────────── */
  const condToggle = document.getElementById('cond-toggle');
  const condEditor = document.getElementById('cond-editor');
  if (condToggle && condEditor) {
    condToggle.addEventListener('click', () => {
      const open = condEditor.hasAttribute('hidden');
      if (open) condEditor.removeAttribute('hidden');
      else condEditor.setAttribute('hidden', '');
      condToggle.setAttribute('aria-expanded', String(open));
      if (open) {
        const first = condEditor.querySelector('select, input');
        if (first) first.focus();
      }
    });
  }

  /* ─────────── 密度切换 ─────────── */
  const segs = document.querySelectorAll('[data-density]');
  const scroll = document.querySelector('.grid-scroll');
  if (segs.length && scroll) {
    const KEY = 'ayanami.grid.density';
    const apply = (mode) => {
      scroll.classList.toggle('grid-comfort', mode === 'comfort');
      scroll.classList.toggle('grid-dense', mode !== 'comfort');
      segs.forEach((b) => {
        const on = b.dataset.density === mode;
        b.classList.toggle('active', on);
        b.setAttribute('aria-pressed', String(on));
      });
      try { localStorage.setItem(KEY, mode); } catch (e) { /* 私密模式忽略 */ }
    };
    let saved = 'dense';
    try { saved = localStorage.getItem(KEY) || 'dense'; } catch (e) { /* 同上 */ }
    apply(saved === 'comfort' ? 'comfort' : 'dense');
    segs.forEach((b) => b.addEventListener('click', () => apply(b.dataset.density)));
  }

  /* ─────────── 批量选择 ─────────── */
  const grid = document.querySelector('[data-grid]');
  if (!grid) return;

  const picks = () => Array.from(grid.querySelectorAll('.row-pick'));
  const selectAll = grid.querySelector('[data-select-all]');
  const modeSwitch = document.getElementById('batch-mode');
  const dock = document.getElementById('batch-dock');
  const dockCount = dock && dock.querySelector('[data-dock-count]');
  const dockIds = dock && dock.querySelector('[data-dock-ids]');
  const clearBtn = dock && dock.querySelector('[data-dock-clear]');
  const submitBtn = dock && dock.querySelector('[data-dock-submit]');

  const selected = () => picks().filter((c) => c.checked);

  const sync = () => {
    const sel = selected();
    const all = picks();
    if (selectAll) {
      selectAll.checked = all.length > 0 && sel.length === all.length;
      selectAll.indeterminate = sel.length > 0 && sel.length < all.length;
    }
    all.forEach((c) => {
      const row = c.closest('tr');
      if (row) row.classList.toggle('is-sel', c.checked);
    });
    if (dockCount) dockCount.textContent = String(sel.length);
    if (dockIds) {
      dockIds.textContent = '';
      sel.slice(0, 3).forEach((c) => {
        const code = document.createElement('code');
        code.textContent = c.value;
        dockIds.appendChild(code);
      });
      if (sel.length > 3) {
        const more = document.createElement('span');
        more.className = 'dock-sum';
        more.textContent = `+${sel.length - 3} 根`;
        dockIds.appendChild(more);
      }
    }
    if (dock) dock.hidden = sel.length === 0;
    if (submitBtn) submitBtn.disabled = sel.length === 0;
  };

  grid.addEventListener('change', (ev) => {
    const t = ev.target;
    if (!(t instanceof HTMLInputElement) || t.type !== 'checkbox') return;
    if (t === selectAll) {
      picks().forEach((c) => { c.checked = t.checked; });
    }
    sync();
  });

  if (clearBtn) {
    clearBtn.addEventListener('click', () => {
      picks().forEach((c) => { c.checked = false; });
      sync();
    });
  }

  const setBatch = (on) => {
    grid.classList.toggle('batch-on', on);
    if (!on) {
      picks().forEach((c) => { c.checked = false; });
    } else {
      const first = picks()[0];
      if (first) first.focus();
    }
    sync();
  };

  if (modeSwitch) {
    setBatch(modeSwitch.checked);
    modeSwitch.addEventListener('change', () => setBatch(modeSwitch.checked));
  }

  document.addEventListener('keydown', (ev) => {
    if (ev.key !== 'Escape') return;
    if (modeSwitch && modeSwitch.checked && grid.classList.contains('batch-on')) {
      modeSwitch.checked = false;
      setBatch(false);
    }
  });

  if (submitBtn) {
    submitBtn.addEventListener('click', (ev) => {
      const n = selected().length;
      if (n === 0) { ev.preventDefault(); return; }
      if (!window.confirm(`确认批量出库选中的 ${n} 根管子？此操作会整批成功或整批失败。`)) {
        ev.preventDefault();
      }
    });
  }

  sync();
})();
