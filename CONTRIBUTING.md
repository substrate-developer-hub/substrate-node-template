# Subtensor Contributor Guide

## Lifecycle of a Pull Request

1. Individuals wishing to contribute to subtensor should develop their change/feature/fix in a
   [Pull Request](https://github.com/opentensor/subtensor/compare) (PR) targeting the `main`
   branch of the subtensor GitHub repository. It is recommended to start your pull request as a
   draft initially until you are ready to have other developers actively look at it. Any
   changes to pallet/runtime code should be accompanied by integration and/or unit tests fully
   testing all the edge cases of this functionality, if applicable.
2. Once you have finished developing your change/feature/fix and the Rust portion of the CI is
   passing for your PR (everything prefixed with "CI"), you should mark your PR as "Ready for
   Review" and request review from "Nucleus".
3. Core Nucleus team members will review your PR, possibly requesting changes, and will also
   add appropriate labels to your PR as shown below. Three positive reviews are required.
4. Once the required passing reviews have been obtained, you are ready to request that your PR
   be included in the next `devnet` deploy. To do this, you should open a companion PR merging
   a copy of your branch into the `devnet-ready` branch. You must include a link to the parent
   PR in the description and preface your PR title with "(Devnet Ready)" or the PR will be
   closed/ignored. Your companion PR should have the `devnet-companion` label.
5. A core team administrator will review your "(Devnet Ready)" PR, verifying that it logically
   matches the changes introduced in the parent PR (there will sometimes be minor differences
   due to merge conflicts) and will either request changes or approve the PR and merge it. Once
   your companion PR is merged, the administrator will add the `devnet-ready` label to the
   parent PR, indicating that the PR is on the `devnet-ready` branch and will be included in
   the next deploy to `devnet`.
6. At some point, a core team administrator will open a PR merging the current `devnet-ready`
   branch into `devnet`, and the CI will enforce some additional safety checks on this PR
   including a requirement that the new `spec_version` be greater than the current on-chain
   `spec_version`. The PR should include a bulleted list of all PRs included in the deploy so
   they can be easily found after the fact (TODO: automate this). This PR will require two
   reviews from the core team as a sanity check. After merging, the administrator will then
   need to update all PRs with the `devnet-ready` label to instead have the `on-devnet` label
   (TODO: automate this upon merge). The administrator will then deploy `devnet`.
7. Once the `on-devnet` label appears on your PR, if you are a core team member it is your
   responsibility to verify that the features/changes/fixes introduced by your PR are
   functioning properly on `devnet` by interacting with the live network. If you are an
   external contributor, a core team member will be assigned to test this for you.
8. If your feature/change/fix is confirmed working on `devnet`, the `devnet-pass` label should
   be added. Otherwise if there are issues, the `devnet-fail` label should be added and you
   will need to make changes to your PR and repeat the previous steps in this process. In some
   cases a revert PR will need to be created reverting your changes from the `pre-devnet` and
   `devnet` branches, respectively.
9. Once `devnet-pass` has been added to your PR, it is eligible for inclusion in the next
   `testnet` deploy. We typically run `testnet` deploys every other wednesday.
10. On the appropriate date, an administrator will open a PR merging the current `devnet`
    branch into `testnet`. This PR should include a bulleted list of all PRs included in the
    deploy so they can be easily found after the fact (TODO: automate this). The PR should
    exclude any PRs that currently have the `devnet-fail` label via a revert (TODO: enforce via
    CI). This PR will require two reviews from the core team as a sanity check. After merging
    into `testnet`, the administrator will then need to run the deploy and update all PRs
    included in the deploy with the `on-testnet` label (TODO: automate this upon merge). Next
    the administrator must cut a (pre-release) release in GitHub for `testnet` (TODO: github
    action to generate the release and release notes).
11. Once the `on-testnet` label appears on your PR, if you are a core team member it is your
    responsibility to once again verify that the features/changes/fixes introduced by your PR
    are functioning properly on `testnet` by interacting with the live network, if applicable.
    If you are an external contributor, a core team member may be assigned to do this testing
    for you but otherwise it will be your responsibility to show evidence on the PR that the
    testing is successful. Once this has been verified, the `testnet-pass` label should be
    added. If testing fails, the `testnet-fail` label should be added and PRs should be opened
    reverting the change from `devnet-ready`, and then a PR should be opened merging the
    modified `devnet` into `testnet`. These revert PRs, if they occur, _must_ be merged before
    a new deploy can be run (TODO: enforce this via CI).
12. After the SOP period (1 week on `testnet`) has passed and the `testnet-pass` label has been
    added, the CI checks on your PR should now turn all green and a core team member will be
    able to merge your PR into `main`. At this point your PR is done and is eligible to be
    included in the next `finney` deploy (TODO: track and enforce SOP compliance on a per-PR
    basis in CI based on the timestamps of label changes). We typically run `finney` deploys
    every other Wednesday, so this will typically happen the Wednesday following the Wednesday
    your PR was deployed to `testnet`. An administrator will run this deploy. The process the
    administrator follows is to open a PR merging `main` into the `finney` branch, which will
    always track the current state of `finney`. This PR automatically has some additional
    checks on it such as asserting that the spec_version gets bumped properly and other sanity
    checks designed to stop a bad deploy. Once the PR is reviewed and merged, the administrator
    will run the actual deploy. Once that is successful, the administrator will cut a new
    GitHub release tagged off of the latest `main` branch commit that was included in the
    deploy, and announcements will be made regarding the release.

## PR Labels

| Name  | Description | Automations |
| ----- | ----------- | ----------- |
| `red-team` | PR is focused on feature additions/changes | none |
| `blue-team` | PR is focused on preventative/safety measures and/or dev UX improvements | none |
| `runtime` | PR contains substantive changes to runtime / pallet code | none |
| `breaking-change` | PR requires synchronized changes with bittensor | Triggers an automatic bot message so the relevant teams are made aware of the change well in advance |
| `migration` | PR contains one or more migrations | none |
| `devnet-companion` | Designates a devnet companion PR | Presence of `devnet-companion` label is checked |
| `devnet-ready` | PR's branch has been merged into the `devnet-ready` branch and will be included in the next `devnet` deploy | none |
| `on-devnet` | PR has been deployed to `devnet` | Removes `devnet-ready` |
| `devnet-pass` | PR has passed manual testing on `devnet` | `devnet-pass` or `devnet-skip` required |
| `devnet-skip` | Allows a critical hotfix PR to skip required testing on `devnet` | `devnet-pass` or `devnet-skip` required |
| `devnet-fail` | PR has failed manual testing on `devnet` and requires modification | none |
| `testnet-companion` | Designates a testnet companion PR | Presence of `testnet-companion` label is checked |
| `on-testnet` | PR has been deployed to `testnet` | none |
| `testnet-pass` | PR has passed manual testing on `testnet` | `testnet-pass` or `testnet-skip` required |
| `testnet-skip` | Allows a critical hotfix PR to skip required manual testing and SOP on `testnet` | `testnet-pass` or `testnet-skip` required |
| `testnet-fail` | PR has failed manual testing on `testnet` and requires modification | none |


## Branches


### `devnet-ready`

Companion PRs merge into this branch, eventually accumulating into a merge of `devnet-ready`
into `devnet`, coinciding with a deploy of `devnet`.

#### Restrictions
* no deleting the branch
* no force pushes
* no direct pushes
* require 1 positive review from an administrator
* new code changes invalidate existing reviews
* only merge commit style merging allowed

#### CI-Enforced Restrictions
* `check-rust.yml` must pass
* TODO: parent PR must be linked to in description
* TODO: parent PR must have the required number of positive reviews


### `devnet`

Tracks the current state of what is deployed to `devnet`. Modified by an administrator via a PR
merging `devnet-ready` into `devnet`, in concert with a deploy of `devnet`.

#### Restrictions
* no deleting the branch
* no force pushes
* no direct pushes
* require 2 positive reviews from core team members
* new code changes invalidate existing reviews
* only merge commit style merging allowed

#### CI-Enforced Restrictions
* `check-rust.yml` must pass
* `check-devnet.yml` must pass
* spec_version must be greater than what is currently on live `devnet`
* TODO: other pre-deploy sanity checks here


### `testnet`

Tracks the current state of what is deployed to `testnet`. Administrator will open a PR merging
current `devnet` into `testnet` and merge it in concert with a deploy to `testnet`. Contains
tags for `testnet` releases.

#### Restrictions
* no deleting the branch
* no force pushes
* no direct pushes
* require 2 positive reviews from core team members
* new code changes invalidate existing reviews
* only merge commit style merging allowed

#### CI-Enforced Restrictions
* `check-rust.yml` must pass
* `check-testnet.yml` must pass
* spec_version must be greater than what is currently on live `testnet`
* TODO: other pre-deploy sanity checks here


### `main`

Default branch for all new PRs. Slightly ahead of what is currently on `finney`. When a PR is all
green and "done", meaning it has been tested on `devnet` and `testnet`, it can be merged into
`main`. Contains tags for `finney` releases.

#### Restrictions
* no deleting the branch
* no force pushes
* no direct pushes
* require 3 positive reviews from core team members
* new code changes invalidate existing reviews
* all conversations must be resolved
* only merge commit style merging allowed

#### CI-Enforced Restrictions
* `check-rust.yml` must pass
* `check-labels.yml` must pass
* must have `devnet-skip` or `devnet-pass` label
* must have `testnet-skip` or `testnet-pass` label
* if `breaking-change` label is present, bot will message the appropriate teams
* TODO: when we get auditing, presence of `needs-audit` label = require a review from auditor
* TODO: track SOP on PR based on label age


### `finney`

Tracks the current state of what is deployed to `finney` (mainnet). Updated via an
administrator-submitted PR merging `main` into `finney` in concert with a `finney` deploy.

#### Restrictions
* no deleting the branch
* no force pushes
* no direct pushes
* require 3 positive reviews from core team members
* new code changes invalidate existing reviews
* only merge commit style merging allowed

#### CI-Enforced Restrictions
* `check-rust.yml` must pass
* `check-finney.yml` must pass
* spec_version must be greater than what is currently on live `finney`
* TODO: other pre-deploy sanity checks here
