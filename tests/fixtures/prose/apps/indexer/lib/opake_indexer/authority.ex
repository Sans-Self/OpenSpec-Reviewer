# Excerpt of Opake's apps/indexer/lib/opake_indexer/authority.ex: the
# moduledoc paragraph whose citation wraps inside a backtick span.

  @doc """
  Validate that a superseding record does not flip its lineage.

  A supersede's declared `lineage` MUST equal its predecessor's lineage
  anchor — the predecessor's own `lineage` if it carries one, otherwise
  the predecessor's own URI (the anchor rule, `spec:lineage § Lineage is
  the chain's genesis URI, carried on every supersede`). This holds the
  whole chain to a single genesis identity: a record that names a
  different lineage is not part of this chain and must not advance it.
