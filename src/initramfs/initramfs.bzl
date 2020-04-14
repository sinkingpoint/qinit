"""
Builds an initramfs from a given set of libraries and binaries
"""

def to_path(f):
    return f.path

def _initramfs_impl(ctx):
    args = ctx.actions.args()

    args.add_all(
        '--bins',
        ctx.files.bins,
        map_each=to_path
    )

    args.add_all(
        '--libs',
        ctx.files.libs,
        map_each=to_path
    )

    args.add('--init', ctx.files.init_script[0].path)
    args.add('--output', ctx.outputs.out)
    args.add('--compress')

    ctx.actions.run(
        mnemonic = "PackageInitRAMFS",
        inputs = ctx.files.libs + ctx.files.bins + ctx.files.init_script,
        executable = ctx.executable.build_initramfs,
        arguments = [args],
        outputs = [ctx.outputs.out],
        env = {
            "LANG": "en_US.UTF-8",
            "LC_CTYPE": "UTF-8",
            "PYTHONIOENCODING": "UTF-8",
            "PYTHONUTF8": "1",
        },
        use_default_shell_env = True,
    )

initramfs = rule(
    implementation = _initramfs_impl,
    attrs = {
        "bins": attr.label_list(allow_files=True, mandatory=True),
        "libs": attr.label_list(allow_files=True, mandatory=True),
        "init_script": attr.label(allow_single_file = True, mandatory=True),
        "out": attr.output(mandatory = True),
        "build_initramfs": attr.label(
            default = Label("//src/initramfs:build_initramfs"),
            cfg = "host",
            executable = True,
            allow_files = True,
        ),
    }
)