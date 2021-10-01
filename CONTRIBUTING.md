# Contributing to Slashcloud
- Code MUST compile on x86 and ARM for both Windows and Linux for testing and command JSON generation
- Code will be deployed to Cloudflare under the wasm32-unknown-unknown triple
    - Note that Cloudflare has a 1 MB size limit on WASM modules. This must be shared between slashcloud and any bot code,
      so features that will consume large amounts of code space (think API type definitions) should be guarded with feature flags
      
## Wanted contributions:
> - [x] Bug fixes in the code.
> - [x] Documenting things.
> - [x] Released features/changes that are not documented.
> - [x] Fixing invalid/outdated example snippets in the documentation.
> - [x] Fixing of spelling, grammatical errors and incorrect statements or inaccuracies in the documentation.

## Unwanted contributions:
> - [ ] Removing whitespace.
> - [ ] Enhancements unrelated to the wrapper.

## Important Notes
- Major feature contributions *MUST* be accompanied by an issue labeled `RFC`. Feature contributions without an RFC will be rejected.
- Some contributions that are wanted or unwanted are (maybe) not mentioned here.
  Maintainers reserve the right to reject your contributions and/or issues
