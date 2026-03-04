# rustup shell setup
if not contains "/home/rkrsn/workspace/scarfbench-org/scarf/.cargo/bin" $PATH
    # Prepending path in case a system-installed rustc needs to be overridden
    set -x PATH "/home/rkrsn/workspace/scarfbench-org/scarf/.cargo/bin" $PATH
end
