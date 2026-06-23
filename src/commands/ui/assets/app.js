document.addEventListener('alpine:init', () => {
    Alpine.data('platys', () => ({
        selected: null,   // the service whose properties we're editing
        openDrawer(svc) {
            this.selected = svc;
        },
        closeDrawer() {
            this.selected = null;
        },
        loading: true,
        services: [],
        grouped: {},
        categories: [],     // all category names (sorted)
        selectedCats: [],    // categories currently shown
        search: '',
        catMenuOpen: false,
        generating: false,
        result: null,     // { success, message } from /api/generate
        previewYaml: '',
        previewOpen: false,
        previewing: false,
        enabledOnly: false, // to display only enabled services
        _snapshot: '',

        async init() {
            const res = await fetch('/api/services');
            const data = await res.json();
            this.services = data.services;
            this.group();
            this.selectedCats = [...this.categories]; // start with everything shown
            this.loading = false;
            this._snapshot = this._stateKey();
        },

        _stateKey() {
            return JSON.stringify(this.services.map(s => ({
                n: s.name,
                e: s.enabled,
                p: s.properties.map(p => p.value),
            })));
        },

        get isDirty() {
            return this._snapshot !== '' && this._stateKey() !== this._snapshot;
        },

        group() {
            const groups = {};
            for (const svc of this.services) {
                const cat = svc.category || 'Other';
                (groups[cat] ||= []).push(svc);
            }
            this.grouped = groups;
            this.categories = Object.keys(groups).sort();
        },

        matches(svc) {
            const q = this.search.trim().toLowerCase();
            if (!q) return true;
            return (svc.display_name || '').toLowerCase().includes(q)
                || (svc.name || '').toLowerCase().includes(q);
        },

        servicesIn(cat) {
            return this.grouped[cat].filter(
                svc => this.matches(svc) && (!this.enabledOnly || svc.enabled)
            );
        },

        // categories that are both selected AND have at least one search match
        get visibleCategories() {
            return this.categories.filter(
                cat => this.selectedCats.includes(cat) && this.servicesIn(cat).length > 0
            );
        },

        toggleCat(cat) {
            const i = this.selectedCats.indexOf(cat);
            if (i === -1) this.selectedCats.push(cat);
            else this.selectedCats.splice(i, 1);
        },

        allCats() {
            this.selectedCats = [...this.categories];
        },
        clearCats() {
            this.selectedCats = [];
        },
        async generate() {
            this.generating = true;
            this.result = null;
            try {
                const payload = {
                    platform_name: null,
                    services: this.services.map(svc => ({
                        name: svc.name,
                        enabled: svc.enabled,
                        properties: Object.fromEntries(svc.properties.map(p => [p.key, p.value])),
                    })),
                };
                const res = await fetch('/api/generate', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify(payload),
                });
                this.result = await res.json();
                if (this.result.success) this._snapshot = this._stateKey();
            } catch (e) {
                this.result = {success: false, message: String(e)};
            } finally {
                this.generating = false;
            }
        },

        async preview() {
            this.previewing = true;
            try {
                const payload = {
                    platform_name: null,
                    services: this.services.map(svc => ({
                        name: svc.name,
                        enabled: svc.enabled,
                        properties: Object.fromEntries(svc.properties.map(p => [p.key, p.value])),
                    })),
                };
                const res = await fetch('/api/preview', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify(payload),
                });
                if (!res.ok) {
                    this.result = {success: false, message: 'Preview failed'};
                    return;
                }
                const data = await res.json();
                this.previewYaml = data.yaml;
                this.previewOpen = true;
            } catch (e) {
                this.result = {success: false, message: String(e)};
            } finally {
                this.previewing = false;
            }
        },

        get enabledCount() {
            return this.services.filter(s => s.enabled).length;
        },

        isBool(p) {
            return p.is_bool;
        },
    }));
});