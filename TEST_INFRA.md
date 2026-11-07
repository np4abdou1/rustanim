# E2E Test Infrastructure Specification: `rustanim`

## 1. Test Philosophy & Design Principles

The `rustanim` end-to-end test suite is built on three core pillars:

1. **Opaque-Box Requirement-Driven Verification**:
   Tests are derived directly from the authoritative specifications mined from `Actions_and_Stuff_112.mcpack`, Minecraft Bedrock / Java humanoid model specifications, and `ORIGINAL_REQUEST.md`. Tests validate external contracts, mathematical invariants, and observable behavior without coupling to private implementation details.

2. **Interface-Compatible Progressive Testability**:
   The test suite targets public API contracts defined in `PROJECT.md` § Interface Contracts:
   - `rustanim::mcpack` archive extraction and resource streaming.
   - `rustanim::anim` Bedrock JSON parsing, Catmull-Rom spline curve sampling, and clip evaluation.
   - `rustanim::molang` zero-allocation bytecode VM, degree trigonometry, and query context.
   - `rustanim::rig` vanilla Java joint anchor preservation, virtual waist rigging, and local matrix math.
   - `rustanim::attachable` dual-perspective 1st-person viewmodel projection and 3rd-person locator attachment.

3. **High Mathematical & Behavioral Fidelity**:
   Expected values are mathematically derived from first principles:
   - Bedrock degree trigonometry ($math.sin(90^\circ) = 1.0$) rather than radians.
   - Cubic Hermite Catmull-Rom splines with non-uniform interval velocity scaling and chord reflection tangents.
   - Hip collapse and limb stretching immunity: verifying that legs maintain independent root anchors ($X = \pm 1.9, Y = 12.0$) when waist tilts or translates.
   - Viewmodel near-plane clipping elimination: verifying that vertex transformations through the dedicated $70^\circ$ FOV, $z_{\text{near}} \le 0.01\text{m}$ matrix maintain valid positive depth.

---

## 2. Feature Inventory to Test Tier Mapping (All 21 Features)

| Feature # | Feature Name | Primary Tier | Description & Test Objectives |
|:---:|---|:---:|---|
| **1** | Mcpack Zip Archive Reader | **Tier 1, Tier 4** | Extract and stream JSON assets directly from `Actions_and_Stuff_112.mcpack` |
| **2** | Bedrock Animation JSON Parser | **Tier 1, Tier 4** | Deserialize `format_version` 1.8.0 / 1.10.0 animation clips, channels, and bones |
| **3** | Catmull-Rom Spline Evaluator | **Tier 1, Tier 2** | Cubic Hermite curve evaluation, interval velocity scaling, asymmetric `pre`/`post` |
| **4** | Animation Clip Sampling | **Tier 1, Tier 3** | Sample bone rotation/translation/scale across timeline with loop and hold modes |
| **5** | Molang Lexer & Tokenizer | **Tier 1, Tier 2** | Tokenize statements, operators, math functions, queries, and variable scopes |
| **6** | Molang Bytecode Compiler & VM | **Tier 1, Tier 3** | Fixed-register execution with zero heap allocations during frame evaluation |
| **7** | Molang Syntax Features | **Tier 1, Tier 2** | Multi-statement blocks (`;`), `return`, conditionals (`? :`), null-coalesce (`??`) |
| **8** | Molang Math Builtins (Degrees) | **Tier 1, Tier 2** | 35 math functions, degree trig, Penner easing curves (`ease_out_cubic`, etc.) |
| **9** | Molang Query & Variable Context | **Tier 1, Tier 3** | Binding queries (`anim_time`, `is_sneaking`, `ground_speed`) and variables (`v.*`, `t.*`, `c.*`) |
| **10** | Vanilla Java Humanoid Anchors | **Tier 1, Tier 3** | Canonical base pivots: Head [0,24,0], Body [0,24,0], Arms [±5,22,0], Legs [±1.9,12,0] |
| **11** | Virtual Waist Joint & Hierarchy | **Tier 1, Tier 3** | Intermediate waist node [0,12,0] rotating body/arms while legs stay on root |
| **12** | Relative Pivot Rigging Formula | **Tier 1, Tier 3** | $M_{\text{local}} = \text{Translate}(A_i - A_p + \Delta T_i) \cdot \text{Rotate}(R_i) \cdot \text{Scale}(S_i)$ |
| **13** | Attachable Definition Parser | **Tier 1, Tier 4** | Parse `attachables/*.json` (`minecraft:attachable` schema: id, materials, scripts) |
| **14** | 1st-Person Viewmodel Matrix | **Tier 1, Tier 2** | Dedicated projection matrix ($70^\circ$ FOV, $z_{\text{near}} \le 0.01\text{m}$) preventing clipping |
| **15** | 3rd-Person Mount Point Rigging | **Tier 1, Tier 4** | Attachable socket binding (`rightitem`, `leftitem`, `head`, `body`) to player bones |
| **16** | Dual-Perspective Context Switch | **Tier 1, Tier 3** | Dynamic switching between 1st-person and 3rd-person transforms via `c.is_first_person` |
| **17** | Standalone Viewer / Binary Demo | **Tier 4** | Playback pipeline integration verified via headless clip execution |
| **18** | GitHub Actions CI Workflows | **Tier 1-4** | Test suite execution in automated CI matrix |
| **19** | Progress Tracking & Git Push Automation | **Tier 4** | Validates item completions matching `tools/progress.py` categories |
| **20** | E2E Requirement-Driven Test Suite | **Tier 1-4** | Comprehensive 4-tier test architecture covering all specifications |
| **21** | Adversarial Coverage Hardening | **Tier 2** | Edge cases, division by zero, extreme angles, step discontinuities |

---

## 3. Test Suite Architecture

```
tests/
├── common/
│   └── mod.rs                         # Shared test fixtures, reference math, mcpack reader
├── e2e_tier1_feature_coverage.rs      # Tier 1: Happy-path feature coverage
├── e2e_tier2_boundary_corner.rs       # Tier 2: Boundary conditions, division by zero, extreme angles
├── e2e_tier3_pairwise_combinations.rs # Tier 3: Pairwise feature interactions & compounding transforms
└── e2e_tier4_real_world_workloads.rs  # Tier 4: Real-world mcpack integration & dual-perspective workloads
```

### Tier Structure Details

1. **Tier 1: Feature Coverage (`tests/e2e_tier1_feature_coverage.rs`)**:
   - Primary behavior (happy path) for all 5 core modules:
     - `mcpack`: archive detection, listing, file extraction.
     - `anim`: JSON clip deserialization, Catmull-Rom spline evaluation with Hermite basis.
     - `molang`: degree trigonometry, math builtins, ternary operators, query context.
     - `rig`: vanilla Java joint anchors, virtual waist node, relative local matrix calculation.
     - `attachable`: item transforms, 1st-person viewmodel projection ($70^\circ$ FOV, $z_{\text{near}}=0.01$), 3rd-person locator mounting.

2. **Tier 2: Boundary & Corner Cases (`tests/e2e_tier2_boundary_corner.rs`)**:
   - Zero-length animations ($L = 0.0\text{s}$) and out-of-range timestamps ($t < 0$, $t > L$).
   - Extreme rotation angles ($\pm 360^\circ$, $\pm 720^\circ$, $1080^\circ$, near-gimbal-lock $\pm 90^\circ$ pitch).
   - Empty bone channels and empty clip maps.
   - Catmull-Rom edge cases: 1-point and 2-point curves, chord reflection tangents ($p_{-1} = 2p_0 - p_1$).
   - Division by zero in Molang (`1 / 0`, `math.mod(x, 0)`) and zero scaling (`scale: 0`, `[0, 0, 0]`).
   - Near-plane clipping boundaries: verifying vertices at $z = -0.02\text{m}$ do not clip through $z_{\text{near}} = 0.01\text{m}$.
   - Discontinuous step keyframes ($pre \neq post$).
   - Numeric bone identifiers (`8dgo1`, `8dgo2`).

3. **Tier 3: Pairwise Combinations (`tests/e2e_tier3_pairwise_combinations.rs`)**:
   - Molang expressions inside animation keyframe channels driving bone rotations and translations.
   - Attachables mounted to dynamic animated humanoid bone chains (root -> waist -> body -> rightarm -> rightitem -> sword).
   - 1st-person viewmodel projection compounded with animated hand swing transformations.
   - Dynamic perspective toggling (`c.is_first_person` true $\leftrightarrow$ false) during ongoing animation playback.
   - Multi-channel bone animation hierarchy: verifying leg hip anchors remain fixed at $Y=12.0$ while waist leans forward $30^\circ$ and rotates $45^\circ$.

4. **Tier 4: Real-World Workloads (`tests/e2e_tier4_real_world_workloads.rs`)**:
   - Directly opening and reading `/home/ubuntu/rustani/Actions_and_Stuff_112.mcpack` (33.8MB).
   - Deserializing and evaluating the real locomotion walk cycle `animation.oreville_ans.tldiyl` across all 15 bones.
   - Deserializing the real 3D diamond sword attachable `attachables/gkx.json` (`minecraft:diamond_sword`).
   - Computing simultaneous 1st-person and 3rd-person transformation matrices for `diamond_sword_3d`.
   - End-to-end mathematical verification of hip collapse immunity and viewmodel clipping prevention on authentic pack assets.

---

## 4. Test Execution Guide

Run all E2E test suites:
```bash
cargo test --test e2e_*
```

Run individual tiers:
```bash
cargo test --test e2e_tier1_feature_coverage
cargo test --test e2e_tier2_boundary_corner
cargo test --test e2e_tier3_pairwise_combinations
cargo test --test e2e_tier4_real_world_workloads
```

Run with standard output visible:
```bash
cargo test --test e2e_* -- --nocapture
```
