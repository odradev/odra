# Contributing to Odra

The following is a set of rules and guidelines for contributing to this repo. Please feel free to propose changes to this document in a pull request.

## Submitting issues

If you have questions about how to use Odra, please direct these to the related channels on the [Odra.dev Discord server](https://discord.gg/Mm5ABc9P8k).

### Guidelines
Please search the existing issues first, it's likely that your issue was already reported or even fixed.
* Go to the main page of the repository, click "issues" and type any word in the top search/command bar.
* You can also filter by appending e. g. "state:open" to the search string.
* More info on [search syntax within GitHub](https://help.github.com/articles/searching-issues)

### Branching model
Odra uses release-based branching model:
* There are no `master` and `develop` branches.
* The active branch is the latest released branch on crates.io.
* Development happens on the next release branch, and it should be a target for all pull requests.

## Contributing to Odra

All contributions to this repository are considered to be licensed under MIT License.

Workflow for bug fixes:
* Check open issues and unmerged pull requests to make sure the topic is not already covered elsewhere.
* Fork the repository.
* Do your changes on your fork.
* Make sure to add or update relevant test cases.
* Create a pull request, with a suitable title and description, referring to the related issue.

Workflow for new features or enhancements:
* Check open issues and unmerged pull requests to make sure the topic is not already covered elsewhere.
* Create an issue including all the reasoning for the new feature or enhancement, and engage in discussion.
* Then fork the repo.
* Do your changes on your fork.
* Make sure to add or update relevant test cases.
* Create a pull request, with a suitable title and description, referring to the related issue and the enchancement proposal.
