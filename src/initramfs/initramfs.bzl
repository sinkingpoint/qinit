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

    extras = []
    if ctx.attr.extra_files:
        args.add('--extrafiles')
        for label, path in ctx.attr.extra_files.items():
            for f in label.files.to_list():
                args.add("{}={}".format(f.path, path))
                extras.append(f)

    args.add('--init', ctx.files.init_script[0].path)
    args.add('--output', ctx.outputs.out)
    
    ctx.actions.run(
        mnemonic = "PackageInitRAMFS",
        inputs = ctx.files.libs + ctx.files.bins + ctx.files.init_script + extras,
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

    return OutputGroupInfo(out = [ctx.outputs.out])

initramfs = rule(
    implementation = _initramfs_impl,
    attrs = {
        "bins": attr.label_list(allow_files=True, mandatory=True),
        "libs": attr.label_list(allow_files=True, mandatory=True),
        "init_script": attr.label(allow_single_file = True, mandatory=True),
        "extra_files": attr.label_keyed_string_dict(allow_empty=True, allow_files=True),
        "out": attr.output(mandatory = True),
        "build_initramfs": attr.label(
            default = Label("//src/initramfs:build_initramfs"),
            cfg = "host",
            executable = True,
            allow_files = True,
        ),
    }
)