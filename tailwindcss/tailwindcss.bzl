"""Rules to handle tailwindcss."""

SrcsInfo = provider(
    doc = "SrcsInfo contains source files for a target",
    fields = {
        "srcs": "depset of source files",
    },
)

def _srcs_aspect_impl(target, ctx):
    # Ignore external dependencies.
    if target.label.workspace_root:
        return []
    srcs = []

    # Make sure the rule has a srcs attribute.
    if hasattr(ctx.rule.attr, "srcs"):
        # Iterate through the files that make up the sources and
        # print their paths.
        for src in ctx.rule.attr.srcs:
            srcs.append(src.files)
    if hasattr(ctx.rule.attr, "deps"):
        for dep in ctx.rule.attr.deps:
            if SrcsInfo in dep:
                srcs.append(dep[SrcsInfo].srcs)
    return [SrcsInfo(srcs = depset(transitive = srcs))]

_srcs_aspect = aspect(
    implementation = _srcs_aspect_impl,
    attr_aspects = ["deps"],
)

def _impl(ctx):
    f = ctx.actions.declare_file("%s.css" % ctx.label.name)
    si = ctx.attr.target[SrcsInfo]

    ctx.actions.run(
        outputs = [f],
        inputs = depset([ctx.file.src, ctx.file._tailwindcss_config], transitive = [si.srcs]),
        executable = ctx.executable._tailwindcss,
        arguments = [
            "--input",
            ctx.file.src.path,
            "--output",
            f.path,
            "-m",
        ],
        mnemonic = "TailwindCSS",
    )
    return [DefaultInfo(
        files = depset([f]),
    )]

tailwindcss = rule(
    implementation = _impl,
    attrs = {
        "src": attr.label(
            allow_single_file = [".css"],
            mandatory = True,
        ),
        "target": attr.label(
            mandatory = True,
            aspects = [_srcs_aspect],
        ),
        "_tailwindcss": attr.label(
            default = "@tailwindcss//:tailwindcss-cli",
            executable = True,
            cfg = "exec",
        ),
        "_tailwindcss_config": attr.label(
            default = "//:tailwind.config.js",
            allow_single_file = True,
        ),
    },
)
