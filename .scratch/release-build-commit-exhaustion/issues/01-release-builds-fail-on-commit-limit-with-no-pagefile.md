# 01: Release builds fail with "LLVM ERROR: out of memory" because this machine has no pagefile and its commit limit is exhausted

**Status:** ready-for-human

**Superseded framing:** this ticket was first written as *"rustc crashes at opt-level 3"* and that was
wrong. Recorded here rather than deleted, because the wrong version would have sent someone splitting
the desktop crate to fix a machine configuration problem. What actually happened is below.

## What is wrong

`cargo build --release -p snapdown-desktop` fails intermittently with `rustc-LLVM ERROR: out of
memory`, sometimes surfacing as rustc exiting `0xc0000409` (`STATUS_STACK_BUFFER_OVERRUN`). It is not
a compile error and not a rustc bug.

The machine, measured 2026-08-30:

| | |
| --- | --- |
| Free **physical** memory | 32.9 GB of 65.4 GB |
| Commit **limit** | 65.4 GB — identical to physical RAM |
| Commit **available** | **3.0 GB** |
| Pagefile | **none configured** |

With no pagefile, Windows caps the commit limit at the size of RAM. Roughly 62 GB is already
*committed* — reserved, whether or not it is resident — so an allocator asking for a few GB is refused
while a third of the machine's RAM sits free. LLVM reports that refusal as "out of memory", which
reads like the compiler's fault and is not.

### "But there is 32 GB of free RAM" — the objection, and why it is the signature rather than a refutation

Free physical memory and available commit are different resources, and this failure only looks
paradoxical if they are conflated. Verified from two independent sources here (`Win32_PerfRawData_PerfOS_Memory`
and `Win32_OperatingSystem`, which agree to within 8 MB):

| | |
| --- | --- |
| Free physical RAM | 32.7 GB |
| Commit charge | 61.9 GB of a 65.4 GB limit |
| Sum of all process working sets | 37.5 GB |

Processes have **asked for** 61.9 GB while **touching** 37.5 GB. The ~24 GB difference is reserved and
untouched — and Windows must still count it, because the moment a process touches a committed page the
system is obliged to produce it. That obligation needs somewhere to live: RAM **or** a pagefile. With no
pagefile the commit limit is pinned to exactly the size of RAM (65,402 MB here, the same number as total
RAM, which is how you can tell at a glance), so new commits get refused while RAM still looks free,
because most of that RAM is already promised to someone else.

So plentiful free RAM alongside refused allocations is the *hallmark* of commit exhaustion, not evidence
against it. Genuine RAM exhaustion presents differently: the machine thrashes, it does not refuse a
compiler's allocation outright.

One measurement trap while checking this: `Get-Counter '\Memory\Available MBytes'` returned **0**, which
is wrong — the raw perf counter says 32,692 MB. Use `Win32_PerfRawData_PerfOS_Memory` and cross-check
against a second source before drawing conclusions from either.

This explains every confusing observation from the session that found it:

- **Why it looked opt-level-specific.** opt-level 3 failed three times, opt-level 1 succeeded once, so
  opt-level 1 was adopted as a workaround — and then failed too, on the very next build, with the same
  error. Then opt-level 3 succeeded with `CARGO_BUILD_JOBS=1`. Both knobs only move peak commit demand;
  neither addresses the limit. Opt-level was a red herring throughout, which is why this ticket's title
  is now wrong and its first framing is disowned above.
- **Why `RUST_MIN_STACK=134217728` did nothing.** The problem is commit, not thread stack.
- **Why 33 GB free RAM was not enough.** Free physical memory is not the resource being exhausted.
- **Why it is intermittent.** It depends on what else holds commit at that moment.

Top commit holders at the time, and the point is the gap between the two columns:

| Process | Commit | Working set |
| --- | --- | --- |
| `mysqld` | 8.7 GB | 15 MB |
| `SnagitEditor` | 3.0 GB | 2.1 GB |
| `java` (x2) | 2.8 GB | 2.3 GB |
| `handy` | 1.6 GB | 62 MB |

`mysqld` alone reserves 8.7 GB it is not using. The listed processes account for ~22 GB of the ~62 GB
committed, so the rest is spread across many more.

## What to do — the owner's call, not an agent's

0. **`CARGO_BUILD_JOBS=1` builds successfully at the default opt-level 3.** Confirmed after the
   diagnosis above: 2m16s, exit 0, no env-var opt-level override. This is the cheapest mitigation and it
   makes sense of everything — cargo runs several rustc processes in parallel by default, and each one's
   peak commit adds to the others'. Serialising them keeps the peak inside the ~3.5 GB of headroom that
   exists. Slower, but it is the whole build, not a reduced one.
1. **Configure a pagefile.** System-managed is enough. This is the real fix, it is a system setting
   rather than a repo change, and it makes the commit limit stop being pinned to RAM. It is what removes
   the need to remember `CARGO_BUILD_JOBS=1`.
2. **Or free commit before a release build.** Stopping `mysqld` and Snagit Editor recovers ~12 GB of
   commit, which is comfortably more than a rustc invocation needs. Do not let an agent stop `mysqld`
   for you — it is a database that may be mid-transaction.

## What NOT to do

**Do not lower `[profile.release]` opt-level in `Cargo.toml` to work around this.** It was considered
while the diagnosis was still wrong. It would trade the product's optimisation for a symptom on one
machine, permanently and invisibly, and it does not even work — opt-level 1 failed too.

**Do not split the desktop crate on the strength of this ticket.** Crate size makes peak memory larger
and so makes the failure likelier, and there are independent reasons to want the split (`main.rs` is
5,500 lines plus all of `appwindow.slint`'s ~3,600 lines of generated code in one unit; `cargo test
--bin Snapdown` hit the same wall). But splitting is a codebase decision that should be argued on its
own merits, not adopted as a fix for an unconfigured pagefile.

## What is still unknown

`.github/workflows/desktop-ci.yml:72` runs `cargo build --release --workspace` on `windows-latest`, and
the workspace `Cargo.toml` has no `[profile.release]` section, so CI builds at opt-level 3. The last
Desktop CI run was **2026-08-24** (run `32678133382`, `main`, green, 4m23s); everything since — the
Settings screen, the Hotkeys tab, the clipboard-only capture path — is on `eval/mattpocock-skills` and
has never been through CI.

A GitHub runner has its own memory configuration, so this machine's commit exhaustion says nothing
about CI. Whether the grown crate still builds there is genuinely open, and pushing the branch is the
cheapest way to find out.

- [ ] It is stated, with a CI run id as evidence, whether `cargo build --release --workspace` passes on
      `windows-latest` for the current branch

## Two collateral traps, both of which cost time here

Diagnostic noise, not separate bugs. Recorded so the next person does not chase them:

- **An aborted build leaves invalid metadata behind.** The run after a crash failed with `E0786` inside
  Slint's `sp::vtable::new_vref` macro, which reads as an error in generated code and is not:
  `rustc --explain E0786` is *"A metadata file was invalid."* `cargo clean -p snapdown-desktop
  --release` clears it. Do that after every crash before believing the next error.
- **Two `cargo` processes on one target dir invent errors.** A `cargo test` started while a
  `cargo build --release` was still running produced `error: cannot determine resolution for the macro
  ::core::include_bytes`, which is not a real problem with the code. Serialise cargo invocations.

And one `AGENTS.md` already warns about, hit again: a background-task notification reported **exit code
0** for a build that failed, because the command ended in `echo "BUILD_EXIT=$?"` — the script's status
is `echo`'s. Read the echoed value, never the notification's code. This is why the failure was noticed
at all.

## Comments

Raised 2026-08-30 while building a release binary for the owner to test the copy-only capture path
(`.scratch/clipboard-only-capture/`). The binary that test ran against was an opt-level 1 build made
during a lucky window and still carried temporary diagnostic logging; the rebuild after that logging was
removed is what failed repeatedly and exposed the real cause. `target/release/Snapdown.exe` is now a
clean opt-level 3 build (2026-08-30 16:23), verified to contain the `BUG-84` fix and none of the
instrumentation.

One of those failures was not memory at all and is worth naming: `Access is denied (os error 5)`, because
a `Snapdown.exe` launched for the owner's test was still running and locking its own file. `AGENTS.md`
documents this exact trap; it was hit anyway, by the agent that had launched the process.
