# Contributing

## Contents

- [Introduction](#introduction)
- [Code of Conduct](#code-of-conduct)
- [Reporting Bugs and Suggesting Improvements](#reporting-bugs-and-suggesting-improvements)
- [Contribution Workflow](#contribution-workflow)
- [Quality Standards](#quality-standards)
- [Release Process](#release-process)

## Introduction

Hello, and welcome to the contributing guide for Agate!

Agate is mostly maintained in the spare time of contributors, so be patient if it takes a bit longer to respond.
By following this guide you'll make it easier for us to address your issues or incorporate your contributions.

We look forward to working with you!

## Code of Conduct

Please note that this project is released with a [Code of Conduct](./CODE_OF_CONDUCT.md).
By participating in this project you agree to abide by its terms.

## Reporting security issues

If you find a security issue, please disclose it to Johann150 privately, e.g. per [email](mailto:johann+agate@qwertqwefsday.eu). If you know how to fix the issue, please follow the contribution workflow as if you do not use GitHub, regardless of if you actually use it. I.e. patches should also be submitted privately.

An effort will be made to respond to such issues quickly, at least responding with a "read receipt". If you do not hear back anything regarding the security issue within three days, try contacting other maintainers listed in the Cargo.toml file or on crates.io for this crate.

There are no bug bounties. You can not expect any compensation apart from attribution in the changelog and/or for any patches you supply.

## Reporting Bugs and Suggesting Improvements

Bugs (unwanted behaviour) and suggested improvements are tracked as [GitHub issues][github-issues].
Before reporting an issue, please check the following points:

1. The issue is caused by Agate itself and not by how it is used.
  Have a look at the documentation if you are not sure.
  If you cannot connect to Agate via the Internet, please try connecting with a client on the same machine to make sure the problem is not caused by intermediate infrastructure.
1. Your issue has not already been reported by someone else.
  Please look through the open issues in the [issue tracker][github-issues].

When reporting an issue, please add as much relevant information as possible.
This will help developers and maintainers to resolve your issue. Some things you might consider:

* Use a descriptive title.
* State which version you are using (use a version tag like `v2.4.1` or the commit hash).
* If you are using tools provided with agate (like a startup script), please also state that.
* Describe how the problem can be reproduced.
* Explain what exactly is the problem and what you expect instead.

[github-issues]: https://github.com/mbrubeck/agate/issues

## Contribution Workflow

Follow these steps to contribute to the project:

### If you use git but not GitHub:

1. Clone the repository where you want.
1. Make the appropriate changes, meeting all [contribution quality standards](#quality-standards).
1. Update the changelog with any added, removed, changed, or fixed functionality. Adhere to the changelog format.
1. Mail the patches or a pull request to [Johann150](mailto:johann+agate@qwertqwefsday.eu).
    - Patches are prefered for small changes.
    - Pull requests have to contain the repository URL and branch name.
1. You will be notified of any further actions (e.g. requested changes, merged) by the same address you sent from. So please make sure you can receive mail on that address.

### If you use GitHub:

1. Make a fork of the [Agate repository][agate-repo].
1. Within your fork, create a branch for your contribution. Use a meaningful name.
1. Create your contribution, meeting all [contribution quality standards](#quality-standards).
1. Update the changelog with any added, removed, changed, or fixed functionality. Adhere to the changelog format.
1. [Create a pull request][create-a-pr] against the `master` branch of the repository.
1. Once the pull request is reviewed and CI passes, it will be merged.

[agate-repo]: https://github.com/mbrubeck/agate
[create-a-pr]: https://help.github.com/articles/creating-a-pull-request-from-a-fork/

## Quality Standards

Most quality and style standards are checked automatically by the CI build.
Contributions should:

- Separate each **logical change** into its own commit.
- Ensure the code compiles correctly, if you can also run `cargo clippy`.
- Format code with `cargo fmt`.
- Avoid adding `unsafe` code.
  If it is necessary, provide an explanatory comment on any `unsafe` block explaining its rationale and why it's safe.
- Add a descriptive message for each commit.
  Follow [these commit message guidelines][commit-messages].
- Document your pull requests.
  Include the reasoning behind each change, and the testing done.

[commit-messages]: https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html

## Release Process
(This is only relevant if you are a maintainer.)

1. Bump the version number appropriately. (Update `Cargo.lock` too!)
1. Run `cargo package` to make sure everything compiles correctly.
1. Update the changelog with the new version ranges.
1. Update agate's homepage (`content/index.gmi`) with changes to the README and CHANGELOG
1. Add a git tag for the version, e.g. with `git tag v2.4.1`.
1. Push the changelog commit and tag to the repository.
    Upon detecting the push of a tag beginning with "v", CI should start building the prebuilt binaries.
    These binaries will be uploaded to a new draft GitHub release with the same name as the version tag. (You need push access to see it).
1. Run `cargo publish` to publish to [crates.io](https://crates.io/crates/agate).
1. Fill the GitHub release text with the appropriate entries from the changelog.
1. Wait for the binary compilation to finish.
1. Publish the GitHub release.
