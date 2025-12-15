function(ctx) {
  identity: {
    id: ctx.identity.id,
    traits: {
      email: ctx.identity.traits.email,
      first_name: if std.objectHas(ctx.identity.traits, 'first_name') then ctx.identity.traits.first_name else '',
      last_name: if std.objectHas(ctx.identity.traits, 'last_name') then ctx.identity.traits.last_name else '',
    },
    created_at: ctx.identity.created_at,
    updated_at: ctx.identity.updated_at,
  },
}
