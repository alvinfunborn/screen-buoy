/** @param {string} raw */
export function parseCharsetInput(raw) {
  return raw.split(',').map(character => character.trim()).filter(Boolean);
}

/** @param {string} character */
const normalizeKey = character => character.replace(/[a-z]/g, letter => letter.toUpperCase());

/** @param {Pick<import('../../types/config').HintConfig, 'charsets' | 'charset_extra'> | undefined} hint */
export function getHintPrefixConflicts(hint) {
  const first = new Set((hint?.charsets?.[0] ?? []).map(normalizeKey));
  return Array.from(new Set((hint?.charset_extra ?? []).map(normalizeKey)))
    .filter(character => first.has(character));
}

/**
 * Only the last position determines HintLeftKey/HintRightKey. Inspect the
 * user's actual mapping; 1 and apostrophe also have awkward edge/cross-row
 * neighbours on the default keyboard even when both actions are mapped.
 * @param {Pick<import('../../types/config').HintConfig, 'charsets'> | undefined} hint
 * @param {import('../../types/config').KeyboardConfig['map_left_right'] | undefined} mapping
 */
export function getHintSideKeyWarnings(hint, mapping) {
  const rows = hint?.charsets ?? [];
  const last = rows[rows.length - 1] ?? [];
  return Array.from(new Set(last.map(normalizeKey))).filter(character => {
    const neighbours = mapping?.[character];
    return !neighbours?.left || !neighbours?.right || character === '1' || character === "'";
  });
}

/** @param {unknown} error @returns {string[] | undefined} */
export function getPrefixConflictsFromError(error) {
  if (typeof error !== 'object' || error === null) return undefined;
  if ('code' in error && error.code === 'hint_prefix_overlap' && 'characters' in error
    && Array.isArray(error.characters) && error.characters.every(character => typeof character === 'string')) {
    return error.characters;
  }
  return undefined;
}
