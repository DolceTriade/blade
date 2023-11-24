"""Rule to get all transitive descriptors for a proto"""

def _impl(ctx):
    pi = ctx.attr.proto[ProtoInfo]
    return [DefaultInfo(
        files = depset(
            [pi.direct_descriptor_set],
            transitive = [pi.transitive_descriptor_sets],
        ),
    )]

transitive_proto_descriptors = rule(
    implementation = _impl,
    attrs = {
        "proto": attr.label(
            mandatory = True,
            providers = [ProtoInfo],
        ),
    },
)
