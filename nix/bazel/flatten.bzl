"""
Flatten is a helper to take a list of inputs (potentially in many directories), and output into a single directory.
"""

def _flatten_impl(ctx):
    out = ctx.label.name
    if ctx.attr.outdir:
        out = ctx.attr.outdir
    d = ctx.actions.declare_directory(out)
    ctx.actions.run_shell(
        outputs = [d],
        inputs = ctx.files.srcs,
        arguments = [d.path] + [x.path for x in ctx.files.srcs],
        mnemonic = "Flatten",
        command = """
            out=$1
            mkdir -p $out
            shift
            while (( $# )); do
                cp $1 $out
                shift
            done
        """
    )
    return DefaultInfo(files=depset([d]))


flatten = rule(
    implementation = _flatten_impl,
    attrs = {
        "outdir": attr.string(
            default = "",
            doc = "Output directory. If empty, uses the label name.",
        ),
        "srcs": attr.label_list(
            allow_files = True,
            doc = "List of inputs to flatten.",
        ),
    }
)
