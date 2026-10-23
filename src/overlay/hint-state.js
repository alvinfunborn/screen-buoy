// Pure session model shared by the overlay and node:test. Codes identify fixed
// screen positions for this session; later scans may only add unused codes.
export class HintState {
    session = 0;
    active = false;
    prefix = '';
    revision = 0;
    hints = new Map();
    pendingFilter = null;

    begin(session) {
        if (session <= this.session) return false;
        this.session = session;
        this.active = true;
        this.prefix = '';
        this.revision = 0;
        this.hints.clear();
        if (this.pendingFilter?.session === session) {
            const { revision, letters } = this.pendingFilter;
            this.filter(session, revision, letters);
        }
        if (this.pendingFilter?.session <= session) this.pendingFilter = null;
        return true;
    }

    append(session, hints) {
        if (!this.active || session !== this.session) return [];
        const added = [];
        for (const hint of hints) {
            if (this.hints.has(hint.text)) continue;
            this.hints.set(hint.text, hint);
            added.push(hint);
        }
        return added;
    }

    filter(session, revision, letters) {
        if (session > this.session) {
            if (!this.pendingFilter || session > this.pendingFilter.session ||
                (session === this.pendingFilter.session && revision > this.pendingFilter.revision)) {
                this.pendingFilter = { session, revision, letters };
            }
            return false;
        }
        if (!this.active || session !== this.session || revision <= this.revision) return false;
        this.prefix = letters;
        this.revision = revision;
        return true;
    }

    display(code) {
        return {
            visible: this.active && code.startsWith(this.prefix),
            text: code.slice(this.prefix.length),
        };
    }

    hide(session) {
        if (session < this.session) return false;
        this.session = session;
        this.active = false;
        this.prefix = '';
        this.hints.clear();
        if (this.pendingFilter?.session <= session) this.pendingFilter = null;
        return true;
    }
}
