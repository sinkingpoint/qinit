load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

git_repository(
    name = "rules_python",
    remote = "https://github.com/bazelbuild/rules_python.git",
    commit = "4b84ad270387a7c439ebdccfd530e2339601ef27",  # (2019-08-02 or later)
)

git_repository(
    name = "io_bazel_rules_rust",
    commit = "fdf9655ba95616e0314b4e0ebab40bb0c5fe005c",
    remote = "https://github.com/bazelbuild/rules_rust.git",
)

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(
    name = "bazel_skylib",
    sha256 = "eb5c57e4c12e68c0c20bc774bfbc60a568e800d025557bc4ea022c6479acc867",
    strip_prefix = "bazel-skylib-0.6.0",
    url = "https://github.com/bazelbuild/bazel-skylib/archive/0.6.0.tar.gz",
)
load("@io_bazel_rules_rust//:workspace.bzl", "bazel_version")
bazel_version(name = "bazel_version")

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repositories")
rust_repositories()