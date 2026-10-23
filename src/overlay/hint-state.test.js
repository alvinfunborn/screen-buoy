import test from 'node:test';
import assert from 'node:assert/strict';
import { HintState } from './hint-state.js';

const hint = (text, x = 10, y = 20) => ({ text, x, y, z: 1, hint_type: 0 });

test('late coordinates append without changing existing codes or positions', () => {
    const state = new HintState();
    state.begin(1);
    state.append(1, [hint('AA'), hint('AS', 40)]);
    assert.deepEqual(state.append(1, [hint('AA', 999), hint('AD', 80)]), [hint('AD', 80)]);
    assert.deepEqual([...state.hints.values()], [hint('AA'), hint('AS', 40), hint('AD', 80)]);
    state.append(1, []); // A later empty scan cannot remove a screen shortcut.
    assert.equal(state.hints.size, 3);
});

test('new hints honor the entire typed prefix, including extra-character codes', () => {
    const state = new HintState();
    state.begin(1);
    state.filter(1, 1, 'QA');
    state.append(1, [hint('QAS'), hint('QSA'), hint('AA')]);
    assert.deepEqual(state.display('QAS'), { visible: true, text: 'S' });
    assert.equal(state.display('QSA').visible, false);
    assert.equal(state.display('AA').visible, false);
    state.filter(1, 2, 'QAS');
    assert.deepEqual(state.display('QAS'), { visible: true, text: '' });
    state.filter(1, 3, '_removeAllHints');
    state.append(1, [hint('QAD')]);
    assert.equal(state.display('QAD').visible, false);
});

test('Escape and rapid reopen reject all late updates from the old session', () => {
    const state = new HintState();
    state.begin(1);
    state.append(1, [hint('AA')]);
    state.hide(2);
    assert.deepEqual(state.append(1, [hint('AS')]), []);
    assert.equal(state.begin(1), false);
    state.begin(3);
    state.append(3, [hint('AA', 500)]);
    assert.equal(state.hide(2), false);
    assert.equal(state.filter(1, 99, 'S'), false);
    assert.deepEqual(state.append(1, [hint('AD')]), []);
    assert.deepEqual([...state.hints.values()], [hint('AA', 500)]);
    assert.equal(state.prefix, '');
});

test('out-of-order filters and a filter delivered before begin preserve latest input', () => {
    const state = new HintState();
    state.filter(3, 8, 'AS');
    state.filter(3, 7, 'A');
    state.begin(3);
    assert.equal(state.prefix, 'AS');
    assert.equal(state.filter(3, 7, 'A'), false);
    assert.equal(state.filter(3, 8, 'AS'), false);
    assert.equal(state.begin(3), false);
    assert.equal(state.prefix, 'AS');
    state.hide(4);
    state.begin(5);
    assert.equal(state.prefix, '');
});
