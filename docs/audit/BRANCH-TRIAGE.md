# BRANCH-TRIAGE.md — Gate 0 Branch Inventory & Recommendations

## Overview
In accordance with Rule L0.4 (ANTI-BOUCLE) and 00-MISSION.md, this document inventories all pre-existing unmerged agent branches on `origin/` and categorizes each branch into one of three recommended actions:
- **`TO-MERGE`**: Branch contains valuable changes that can be evaluated or integrated.
- **`TO-CLOSE`**: Branch is a superseded, redundant, or orphaned attempt from previous agent runs.
- **`TO-IGNORE`**: Active or special branch that should remain untouched.

> **Note**: As required by §L0.2 and §L0.4, no remote branches have been modified or deleted.

---

## Inventory & Recommendations

| Branch Name | Type / Pattern | Recom. Action | Justification |
| :--- | :--- | :--- | :--- |
| `origin/bump-version-1-0-9-5653294599933949887` | `bump-version-*` | `TO-CLOSE` | Version bump attempt superseded by Gate process. |
| `origin/bump-version-1.0.9-15834995085071634324` | `bump-version-*` | `TO-CLOSE` | Version bump attempt superseded by Gate process. |
| `origin/bump-version-1.0.9-18110830604252788492` | `bump-version-*` | `TO-CLOSE` | Version bump attempt superseded by Gate process. |
| `origin/bump-version-to-1-0-9-7325395898866018307` | `bump-version-*` | `TO-CLOSE` | Version bump attempt superseded by Gate process. |
| `origin/chore/general-fixes-and-optimizations-2913194210888921791` | `chore/*` | `TO-CLOSE` | Redundant optimization attempt. |
| `origin/chore/update-readme-and-bump-version-15311994094188517691` | `chore/*` | `TO-CLOSE` | Readme update superseded by G9 docs update. |
| `origin/docs/update-readme-12975765816275098181` | `docs/*` | `TO-CLOSE` | Readme update superseded by G9 docs update. |
| `origin/feat/add-apply-command-126512599903172527` | `feat/*` | `TO-CLOSE` | `apply` command already merged into `main`. |
| `origin/fix-and-optimize-10361938808671734267` | `fix-*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix-and-optimize-code-9832981319222116124` | `fix-*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix-check-optimize-readme-8780637137578144224` | `fix-*` | `TO-CLOSE` | Readme update attempt. |
| `origin/fix-clippy-and-format-10321038344215275657` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2 (Lint & panic hygiene). |
| `origin/fix-clippy-and-version-4106688597242751368` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-clippy-unwrap-and-readme-13003230908180269500` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-clippy-unwrap-readme-8156432390460564111` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-clippy-update-version-14545971677968259771` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-clippy-warnings-and-readme-12156606936119769779` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-clippy-warnings-and-update-readme-18122866608578674719` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-clippy-warnings-and-update-version-11230526505686671068` | `fix-clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-clone-unwrap-1810436231440212601` | `fix-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-format-10095215607361988454` | `fix-*` | `TO-CLOSE` | Superseded by Gate G1. |
| `origin/fix-hardware-disks-and-optimize-strings-12321768545356847614` | `fix-*` | `TO-CLOSE` | Orphaned fix attempt. |
| `origin/fix-hardware-refresh-unwrap-startup-4645193390253376790` | `fix-*` | `TO-CLOSE` | Hardware disk refresh logic integrated into main. |
| `origin/fix-issues-and-optimize-6387313782639308973` | `fix-*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix-optimizations-7774359018410377599` | `fix-*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix-optimize-readme-10556038123047398579` | `fix-*` | `TO-CLOSE` | Readme update attempt. |
| `origin/fix-optimize-readme-6110476742196598406` | `fix-*` | `TO-CLOSE` | Readme update attempt. |
| `origin/fix-optimize-readme-7860980187908614054` | `fix-*` | `TO-CLOSE` | Readme update attempt. |
| `origin/fix-optimize-readme-version-15139111432701112019` | `fix-*` | `TO-CLOSE` | Readme update attempt. |
| `origin/fix-optimize-rust-string-handling-16644156586020537955` | `fix-*` | `TO-CLOSE` | String handling optimizations merged into main. |
| `origin/fix-option-clone-unwrap-15198918450686983932` | `fix-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-readme-version-and-passwords-13608642135302100912` | `fix-*` | `TO-CLOSE` | Readme/passwords fix attempt. |
| `origin/fix-type-inference-and-version-5240018712907379811` | `fix-*` | `TO-CLOSE` | Type inference fixes already present in main. |
| `origin/fix-unnecessary-allocations-15945483009442826342` | `fix-*` | `TO-CLOSE` | Allocation optimizations already present in main. |
| `origin/fix-unwrap-and-docs-11869832413200673978` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-optimize-hardware-detection-15289885548092091820` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-optimize-readme-6170010143814949727` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-readme-13918624029975592039` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-readme-6070973383431964097` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-10519438907694478062` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-13733841205839268923` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-2101304812704299147` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-3063317215499740514` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-311266833678534829` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-3601462804881267013` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-4981917951870218090` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-7572977343683632745` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-readme-8147267195560085203` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-and-update-version-12475245458105493359` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-clone-and-docs-2156747669472477316` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-logic-optimization-1392812493609958489` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-or-allocations-107598217805566108` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-unwrap-sysinfo-version-bump-5345104748064262939` | `fix-unwrap-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix-version-and-optimize-4067735368012221914` | `fix-*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix-version-bump-1.0.9-1083481205902550833` | `fix-*` | `TO-CLOSE` | Version bump attempt. |
| `origin/fix-web-ui-lints-and-readme-7013671270130306220` | `fix-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/bump-version-1.0.9-12142874407630143721` | `fix/*` | `TO-CLOSE` | Version bump attempt. |
| `origin/fix/check-fix-optimize-10596089835264523320` | `fix/*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix/clippy-and-version-bump-5684201599095675287` | `fix/clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/clippy-unwrap-and-readme-9062783121755827085` | `fix/clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/clippy-warnings-and-readme-3832017407085482203` | `fix/clippy-*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/code-cleanup-and-readme-3281308738009778532` | `fix/*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix/code-quality-and-hardware-bug-3527738620918592042` | `fix/*` | `TO-CLOSE` | Hardware fixes already merged. |
| `origin/fix/hardware-disks-version-18343072356069251569` | `fix/*` | `TO-CLOSE` | Hardware disk fixes already merged. |
| `origin/fix/optimize-and-bump-version-16055920041032478000` | `fix/*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix/optimize-and-bump-version-6227305342329792100` | `fix/*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix/optimize-and-update-readme-10180828590681851955` | `fix/*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix/optimize-and-update-readme-7733571028200050740` | `fix/*` | `TO-CLOSE` | Orphaned agent attempt. |
| `origin/fix/optimize-clone-calls-13358462337472731902` | `fix/*` | `TO-CLOSE` | Clone optimizations merged into main. |
| `origin/fix/optimize-unwrap-or-default-15097100323596521382` | `fix/*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/option-string-optimization-v1.0.9-10628383382632751581` | `fix/*` | `TO-CLOSE` | String optimizations merged into main. |
| `origin/fix/replace-unwrap-with-expect-3425786917330565417` | `fix/*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/replace-unwrap-with-expect-9250986235517503576` | `fix/*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/unwrap-and-optimize-web-and-update-readme-5852911539592984555` | `fix/*` | `TO-CLOSE` | Superseded by Gate G2. |
| `origin/fix/update-readme-clippy-unix-epoch-16907592878872422361` | `fix/*` | `TO-CLOSE` | Orphaned attempt. |
| `origin/fix/update-readme-version-652552791068939725` | `fix/*` | `TO-CLOSE` | Orphaned attempt. |
| `origin/fix/version-and-clone-opts-10228386331237665096` | `fix/*` | `TO-CLOSE` | Orphaned attempt. |
| `origin/jules-*` (30+ branches) | `jules-*` | `TO-CLOSE` | Previous agent session branches; all work superseded by current gate execution. |
| `origin/main` | Primary branch | `TO-IGNORE` | Primary repository branch. |
