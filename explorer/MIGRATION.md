# Migration Guide: Explorer Modular Redesign

## Overview
The FlowCortex Explorer has been restructured with a modern modular architecture. This guide helps you understand the changes.

## What Changed

### Templates
**Before**: Single monolithic `index.html` in templates/
**After**: Organized modular structure:
- Main entry: `templates/index.html`
- Reusable components: `templates/components/*.html`
- Individual pages: `templates/pages/*.html`

### JavaScript Modules
**No Changes** - All JS modules remain in `static/js/`
- `app.js` - still the main app
- `modules/api.js` - API communication (unchanged)
- `modules/wallet.js` - signing (unchanged)
- `modules/charts.js` - visualization (unchanged)
- `modules/ui.js` - UI logic (unchanged)

### Build System
**Before**: `Cargo.toml` with invalid edition "2024"
**After**: Correct edition "2021" + Askama.toml configuration

## File Locations

### Templates That Changed
```
index.html.old          → index.html (new structure)
                        → base.html (base layout)
                        → pages/*.html (individual pages)
                        → components/*.html (reusable components)
```

### New Files Added
```
templates/
├── index.html                    # NEW - Entry point
├── base.html                     # NEW - Base layout
├── components/
│   ├── nav.html                 # NEW
│   ├── modals.html              # NEW
│   ├── stat_card.html           # NEW
│   ├── chart_card.html          # NEW
│   ├── button.html              # NEW
│   ├── input_text.html          # NEW
│   └── input_textarea.html      # NEW
└── pages/
    ├── dashboard.html           # NEW
    ├── balance.html             # NEW
    ├── blocks.html              # NEW
    ├── transactions.html        # NEW
    ├── capsules.html            # NEW
    ├── anchors.html             # NEW
    └── wallet.html              # NEW

Askama.toml                       # NEW - Template config
```

## Functional Changes

### No Breaking Changes! ✅
- All existing functionality preserved
- Same API endpoints
- Same UI features
- Same keyboard shortcuts
- Same dark mode support

### Improvements
1. **Better Organization** - Templates organized by feature
2. **Reusable Components** - Common UI patterns
3. **Easier Maintenance** - Find and edit features faster
4. **Better Scalability** - Easy to add new pages/components
5. **Cleaner Code** - Separation of concerns

## Build Instructions

### Before
```bash
cd explorer
cargo check
cargo run
```

### After
```bash
cd explorer
# Same commands work!
cargo check
cargo run
```

**No changes needed to build process!**

## Upgrading Your Deployment

### Drop-in Replacement
The new version is a drop-in replacement:

```bash
# Backup old version (optional)
git stash

# Pull new modular version
git pull

# Build and run (same as before)
cd explorer
cargo build --release
./target/release/flowcortex-explorer
```

### Database/State
No migration needed - no persistent storage in explorer.

### Configuration
Same environment variables work:
```bash
BIND_ADDR=0.0.0.0:4000  # Still works
```

## Common Tasks

### Adding a New Page

1. Create `templates/pages/new_page.html`
2. Add page logic to `static/js/app.js`
3. Add nav button to `templates/components/nav.html`
4. Build and deploy (no code changes needed)

### Adding a UI Component

1. Create `templates/components/new_component.html`
2. Use in pages where needed
3. No JS changes required

### Modifying Existing Page

```bash
# Find the page
templates/pages/dashboard.html    # Changed? Here it is!

# Edit and rebuild
cargo build --release
```

### Styling Updates

1. Edit `static/css/styles.css`
2. No rebuild needed - style file is served as-is

## Testing

### Verify Installation
```bash
# Build should succeed
cargo build --release

# Should start on port 4000
cargo run

# Should load at http://192.168.29.78:4000
# All pages should work
```

### Verify Functionality
- [ ] Dashboard loads and shows stats
- [ ] Dark mode toggle works
- [ ] Navigation between tabs works
- [ ] All 7 pages are accessible
- [ ] API calls work (need L1 node running)

## Rollback (if needed)

If you need to go back:
```bash
git checkout explorer/templates/index.html.old
# Copy back to index.html and rebuild
```

But everything should work the same! The modular structure is fully backward compatible.

## Questions?

Refer to:
- `README_REDESIGN.md` - Detailed architecture guide
- `README.md` - Updated documentation
- `templates/` - Source code is self-documenting

## Benefits You Get

✅ Cleaner codebase
✅ Faster feature development
✅ Easier bug fixes
✅ Better team collaboration
✅ Prepared for growth
✅ Better performance (no changes)
✅ Same user experience

---

**Migration Status**: ✅ **SEAMLESS**  
**Backward Compatible**: ✅ **YES**  
**Zero Downtime**: ✅ **POSSIBLE**

Enjoy the improved explorer!
