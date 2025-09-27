# Fix All Warnings and Errors TODO List

## Summary
Total Warnings: 279+ (mostly duplicates)
Total Errors: 0

## Categorized Issues to Fix

### 1. ❌ Unused Imports (5 instances)
- [ ] `std::cmp::Ordering` in src/concurrent/executor.rs:8
- [ ] `Sender` (location TBD)
- [ ] `Stdio` (location TBD)
- [ ] `signal` (location TBD)
- [ ] `ratatui::buffer::Cell` (location TBD)

### 2. ❌ Unused Variables (6 instances)
- [ ] `never_chan` in executor.rs
- [ ] `i` (location TBD)
- [ ] `frame` (location TBD)
- [ ] `dns_handle` (location TBD)
- [ ] `bandwidth_graph_state` (location TBD)
- [ ] Variable that doesn't need to be mutable (location TBD)

### 3. ❌ Unused Methods/Functions (4 instances)
- [ ] `progressive_split` method never used
- [ ] `build_two_children_layout` method never used
- [ ] `build_three_children_layout` method never used
- [ ] `build_timer_select` method never used

### 4. ❌ Unused Struct Fields (3 instances)
- [ ] Fields `rx`, `in_use`, and `timer_index` are never read in TimerChannel

### 5. ❌ Unused Constants (2 instances)
- [ ] `FIRST_WIDTH_BREAKPOINT` is never used
- [ ] `FIRST_HEIGHT_BREAKPOINT` is never used

### 6. ❌ Deprecated API Usage (3 instances)
- [ ] `ratatui::prelude::Buffer::get` - use `Buffer[(x, y)]` instead
- [ ] `ratatui::prelude::Buffer::get_mut` - use `Buffer[(x, y)]` instead  
- [ ] `ratatui::Terminal::<B>::set_cursor` - use `set_cursor_position((x, y)]` instead

### 7. ❌ Unreachable Pattern (1 instance)
- [ ] Unreachable pattern (location TBD)

### 8. ❌ Feature Flag Warnings (90+ instances)
- [ ] All `cfg` condition values for spinners features that don't exist in Cargo.toml
  - aesthetic, arc, arrow, arrow2, arrow3, balloon, balloon2, beta_wave, binary, blue_pulse, bounce, bouncing_ball, bouncing_bar, box_bounce, box_bounce2, christmas, circle, circle_halves, circle_quarters, clock, dots, dots2-14, dots_circle, dqpb, dwarf_fortress, earth, finger_dance, fist_bump, flip, grenade, grow_horizontal, grow_vertical, hamburger, hearts, layer, line, line2, material, mindblown, monkey, moon, noise, orange_blue_pulse, orange_pulse, pipe, point, pong, runner, sand, shark, simple_dots, simple_dots_scrolling, smiley, soccer_header, speaker, square_corners, squish, star, star2, time_travel, toggle, toggle2-13, triangle, weather

## Execution Plan

### Phase 1: Analyze and Understand Each Warning
1. Use grep/search to find exact locations of all warnings
2. Understand the context and purpose of each unused item
3. Determine if it should be implemented, removed, or annotated

### Phase 2: Fix in Order of Impact
1. Fix deprecated API usage (affects runtime)
2. Fix unused variables and imports (code cleanliness)
3. Fix unused methods/fields (potential missing functionality)
4. Fix feature flags (build configuration)

### Phase 3: Verification
1. Run `cargo check --all-targets` after each fix
2. Run `cargo test` to ensure no regressions
3. Run the application to verify functionality

## Notes
- Many warnings appear to be duplicated across different build targets
- The spinner feature flags need to be either added to Cargo.toml or the code needs to be restructured
- Some unused items may indicate incomplete implementations that need to be finished