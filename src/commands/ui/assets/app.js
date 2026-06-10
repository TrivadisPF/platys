document.addEventListener('alpine:init', () => {
    Alpine.data('platys', () => ({
        loading: true,
        services: [],
        grouped: {},
        categories: [],     // all category names (sorted)
        selectedCats: [],    // categories currently shown
        search: '',
        catMenuOpen: false,

        async init() {
            const res = await fetch('/api/services');
            const data = await res.json();
            this.services = data.services;
            this.group();
            this.selectedCats = [...this.categories]; // start with everything shown
            this.loading = false;
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
            return this.grouped[cat].filter(svc => this.matches(svc));
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
    }));
});