import test from 'node:test';
import assert from 'node:assert/strict';
import { getHintPrefixConflicts, getHintSideKeyWarnings, getPrefixConflictsFromError, parseCharsetInput } from './hint-validation.js';

test('charset editing keeps comma-separated input as arrays, including an unfinished delimiter', () => {
  assert.deepEqual(parseCharsetInput(' F, D, '), ['F', 'D']);
  assert.deepEqual(parseCharsetInput(" A, Q, Z, 1, =, \\, /, ' "), ['A', 'Q', 'Z', '1', '=', '\\', '/', "'"]);
  assert.deepEqual(parseCharsetInput(''), []);
});

test('only first-position overlap blocks save, with stable unique conflicts ignoring ASCII case', () => {
  assert.deepEqual(getHintPrefixConflicts({ charsets: [['F', 'D'], ['Q', 'A']], charset_extra: ['d', 'Q', 'F', 'D'] }), ['D', 'F']);
  assert.deepEqual(getHintPrefixConflicts({ charsets: [['F', 'D'], ['Q', 'A']], charset_extra: ['Q', 'A'] }), []);
  assert.deepEqual(getHintPrefixConflicts({ charsets: [['F', 'D']], charset_extra: [] }), []);
});

test('side-key advisory follows the final row and user mapping, never the extra prefix', () => {
  const mapping = {
    D: { left: 'S', right: 'F' }, Q: { right: 'W' }, A: { right: 'S' }, Z: { right: 'X' },
    '1': { left: '`', right: '2' }, '=': { left: '-' }, '\\': { left: ']' },
    '/': { left: '.' }, "'": { left: ';', right: '/' },
  };
  const edgeKeys = ['A', 'Q', 'Z', '1', '=', '\\', '/', "'"];
  assert.deepEqual(getHintSideKeyWarnings({ charsets: [edgeKeys, ['D']] }, mapping), []);
  assert.deepEqual(getHintSideKeyWarnings({ charsets: [['D'], [...edgeKeys, 'D', 'a']] }, mapping), edgeKeys);
  assert.deepEqual(getHintSideKeyWarnings({ charsets: [['Q']] }, { Q: { left: 'Tab', right: 'W' } }), []);
  assert.deepEqual(getHintSideKeyWarnings({ charsets: [['D']] }, {}), ['D']);
});

test('backend conflict details are recognized without masking generic save errors', () => {
  assert.deepEqual(getPrefixConflictsFromError({ code: 'hint_prefix_overlap', characters: ['F', 'D'] }), ['F', 'D']);
  assert.equal(getPrefixConflictsFromError({ code: 'save_failed', message: 'disk full' }), undefined);
  assert.equal(getPrefixConflictsFromError('old error format'), undefined);
  assert.equal(getPrefixConflictsFromError({ code: 'hint_prefix_overlap', characters: [1] }), undefined);
});
