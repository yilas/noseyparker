# Nosey Parker: Find secrets in textual data

Nosey Parker is a command-line tool that finds secrets and sensitive information in textual data. It is useful both for offensive and defensive security testing.

**Key features:**
- It supports scanning files, directories, and the entire history of Git repositories
- It uses regular expression matching with a set of 99 patterns chosen for high signal-to-noise based on experience and feedback from offensive security engagements
- It groups matches together that share the same secret, further emphasizing signal over noise
- It is fast: it can scan at hundreds of megabytes per second on a single core, and is able to scan 100GB of Linux kernel source history in less than 2 minutes on an older MacBook Pro

This open-source version of Nosey Parker is a reimplementation of the internal version that is regularly used in offensive security engagements at [Praetorian](https://praetorian.com). The internal version has additional capabilities for false positive suppression and an alternative machine learning-based detection engine. Read more in blog posts [here](https://www.praetorian.com/blog/nosey-parker-ai-secrets-scanner-release/) and [here](https://www.praetorian.com/blog/six-months-of-finding-secrets-with-nosey-parker/).


## Installation

### Prebuilt Binaries

Prebuilt binaries are available for x86_64 Linux and x86_64/ARM64 macOS on the [latest release page](https://github.com/praetorian-inc/noseyparker/releases/latest).
This is the simplest way to get started and will give good performance.

### Docker Image

A prebuilt multiplatform Docker image is available for the latest release for x86_64 and ARM64:

```
docker pull ghcr.io/praetorian-inc/noseyparker:latest
```

A prebuilt Docker image is available for the most recent commit for x86_64:

```
docker pull ghcr.io/praetorian-inc/noseyparker:edge
```

**Note:** The Docker image runs noticeably slower than a native binary, particularly on macOS.

### Building from source

**1. Prerequisites**
This has been tested on several versions of Ubuntu Linux on x86_64 and on macOS running on both Intel and ARM processors.

Required dependencies:
- `cargo`: recommended approach:install from <https://rustup.rs>
- `cmake`: needed for building the `vectorscan-sys` crate
- `git`: needed for embedding version information into the `noseyparker` CLI

**2. Build using [Cargo](https://doc.rust-lang.org/cargo/)**

```
cargo build --release
```
This will produce an optimized binary at `target/release/noseyparker`.

## Usage quick start

**Note:** If you are using the Docker image, replace `noseyparker` in the following commands with a Docker invocation that uses a mounted volume:

```
docker run -v "$PWD":/opt/ ghcr.io/praetorian-inc/noseyparker:latest <ARGS>
```

### The datastore
Most Nosey Parker commands use a _datastore_.
This is a special directory that Nosey Parker uses to record its findings and maintain its internal state.
A datastore will be implicitly created by the `scan` command if needed.
You can also create a datastore explicitly using the `datastore init -d PATH` command.


### Scanning filesystem content for secrets
Nosey Parker has built-in support for scanning files, recursively scanning directories, and scanning the entire history of Git repositories.

For example, if you have a Git clone of [CPython](https://github.com/python/cpython) locally at `cpython.git`, you can scan its entire history with the `scan` command.
Nosey Parker will create a new datastore at `np.cpython` and saves its findings there.
```
$ noseyparker scan --datastore np.cpython cpython.git
Found 28.30 GiB from 18 plain files and 427,712 blobs from 1 Git repos [00:00:04]
Scanning content  ████████████████████ 100%  28.30 GiB/28.30 GiB  [00:00:53]
Scanned 28.30 GiB from 427,730 blobs in 54 seconds (538.46 MiB/s); 4,904/4,904 new matches

 Rule                      Distinct Groups   Total Matches
───────────────────────────────────────────────────────────
 PEM-Encoded Private Key             1,076           1,192
 Generic Secret                        331             478
 netrc Credentials                      42           3,201
 Generic API Key                         2              31
 md5crypt Hash                           1               2

Run the `report` command next to show finding details.
```

### Scanning Git repos by URL, GitHub username, or GitHub organization name
Nosey Parker can also scan Git repos that have not already been cloned to the local filesystem.
The `--git-url URL`, `--github-user NAME`, and `--github-org NAME` options to `scan` allow you to specify repositories of interest.

For example, to scan the Nosey Parker repo itself:
```
$ noseyparker scan --datastore np.noseyparker --git-url https://github.com/praetorian-inc/noseyparker
```

For example, to scan accessible repositories belonging to [`octocat`](https://github.com/octocat):
```
$ noseyparker scan --datastore np.noseyparker --github-user octocat
```

These input specifiers will use an optional GitHub token if available in the `NP_GITHUB_TOKEN` environment variable.
Providing an access token gives a higher API rate limit and may make additional repositories accessible to you.

See `noseyparker help scan` for more details.

### Summarizing findings
Nosey Parker prints out a summary of its findings when it finishes
scanning.  You can also run this step separately:
```
$ noseyparker summarize --datastore np.cpython

 Rule                      Distinct Groups   Total Matches
───────────────────────────────────────────────────────────
 PEM-Encoded Private Key             1,076           1,192
 Generic Secret                        331             478
 netrc Credentials                      42           3,201
 Generic API Key                         2              31
 md5crypt Hash                           1               2
```

Additional output formats are supported, including JSON and JSON lines, via the `--format=FORMAT` option.


### Reporting detailed findings
To see details of Nosey Parker's findings, use the `report` command.
This prints out a text-based report designed for human consumption:
```
$ noseyparker report --datastore np.cpython
Finding 1/1452: Generic API Key
Match: QTP4LAknlFml0NuPAbCdtvH4KQaokiQE
Showing 3/29 occurrences:

    Occurrence 1:
    Git repo: clones/cpython.git
    Blob: 04144ceb957f550327637878dd99bb4734282d07
    Lines: 70:61-70:100

        e buildbottest

        notifications:
          email: false
          webhooks:
            urls:
              - https://python.zulipchat.com/api/v1/external/travis?api_key=QTP4LAknlFml0NuPAbCdtvH4KQaokiQE&stream=core%2Ftest+runs
            on_success: change
            on_failure: always
          irc:
            channels:
              # This is set to a secure vari

    Occurrence 2:
    Git repo: clones/cpython.git
    Blob: 0e24bae141ae2b48b23ef479a5398089847200b3
    Lines: 174:61-174:100

        j4 -uall,-cpu"

        notifications:
          email: false
          webhooks:
            urls:
              - https://python.zulipchat.com/api/v1/external/travis?api_key=QTP4LAknlFml0NuPAbCdtvH4KQaokiQE&stream=core%2Ftest+runs
            on_success: change
            on_failure: always
          irc:
            channels:
              # This is set to a secure vari
...
```

(Note: the findings above are synthetic, invalid secrets.)
Additional output formats are supported, including JSON and JSON lines, via the `--format=FORMAT` option.


### Enumerating repositories from GitHub
To list URLs for repositories belonging to GitHub users or organizations, use the `github repos list` command.
This command uses the GitHub REST API to enumerate repositories belonging to one or more users or organizations.
For example:
```
$ noseyparker github repos list --user octocat
https://github.com/octocat/Hello-World.git
https://github.com/octocat/Spoon-Knife.git
https://github.com/octocat/boysenberry-repo-1.git
https://github.com/octocat/git-consortium.git
https://github.com/octocat/hello-worId.git
https://github.com/octocat/linguist.git
https://github.com/octocat/octocat.github.io.git
https://github.com/octocat/test-repo1.git
```

An optional GitHub Personal Access Token can be provided via the `NP_GITHUB_TOKEN` environment variable.
Providing an access token gives a higher API rate limit and may make additional repositories accessible to you.

Additional output formats are supported, including JSON and JSON lines, via the `--format=FORMAT` option.

See `noseyparker help github` for more details.


### Getting help
Running the `noseyparker` binary without arguments prints top-level help and exits.
You can get abbreviated help for a particular command by running `noseyparker COMMAND -h`.

**Tip: More detailed help is available with the `help` command or long-form `--help` option.**


## Contributing
Contributions are welcome, particularly new regex rules.
Developing new regex rules is detailed in a [separate document](docs/RULES.md).

If you are considering making significant code changes, please [open an issue](https://github.com/praetorian-inc/noseyparker/issues/new) first to start discussion.


## License
Nosey Parker is licensed under the [Apache License, Version 2.0](LICENSE).

Any contribution intentionally submitted for inclusion in Nosey Parker by you, as defined in the Apache 2.0 license, shall be licensed as above, without any additional terms or conditions.

Nosey Parker also includes vendored copies of several other packages released under the Apache License and other permissive licenses; see `LICENSE` for details.
