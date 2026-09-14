# Excerpt of Opake's workspace_controller.ex: a `#` comment that wraps.
  # transient case gets its own machine-readable code and 403 is reserved
  # for a head that was actually consulted.
  #
  # See spec:indexer-consistency § Unknown workspace is distinguishable from
  # non-membership.
  @spec check_membership(String.t(), String.t()) :: :ok | {:error, non_neg_integer(), String.t()}
