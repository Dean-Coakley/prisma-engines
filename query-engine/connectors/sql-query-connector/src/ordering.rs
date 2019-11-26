use connector_interface::OrderDirections;
use prisma_models::*;
use quaint::ast::*;

pub type OrderVec<'a> = Vec<(DatabaseValue<'a>, Option<Order>)>;

pub struct Ordering;

/// Tooling for generating orderings for different query types.
impl Ordering {
    pub fn for_model(
        model: ModelRef,
        order_by: Option<&OrderBy>,
        order_directive: OrderDirections,
    ) -> OrderVec<'static> {
        Self::by_fields(
            order_by.map(|oby| oby.field.as_column()),
            model.fields().id().as_column(),
            order_by,
            order_directive,
        )
    }

    pub fn internal<C>(
        second_field: C,
        order_by: Option<&OrderBy>,
        order_directive: OrderDirections,
    ) -> OrderVec<'static>
    where
        C: Into<Column<'static>>,
    {
        Self::by_fields(
            order_by.map(|oby| oby.field.as_column()),
            second_field.into(),
            order_by,
            order_directive,
        )
    }

    pub fn aliased_internal(
        alias: &str,
        secondary_alias: &str,
        secondary_field: &str,
        order_by: Option<&OrderBy>,
        order_directive: OrderDirections,
    ) -> OrderVec<'static> {
        Self::by_fields(
            order_by.map(|oby| (alias.to_string(), oby.field.db_name().to_string()).into()),
            (secondary_alias.to_string(), secondary_field.to_string()).into(),
            order_by,
            order_directive,
        )
    }

    fn by_fields(
        first_column: Option<Column<'static>>,
        second_column: Column<'static>,
        order_by: Option<&OrderBy>,
        order_directive: OrderDirections,
    ) -> OrderVec<'static> {
        let default_order = order_by
            .as_ref()
            .map(|order| order.sort_order)
            .unwrap_or(SortOrder::Ascending);

        match first_column {
            Some(first) => {
                if first != second_column {
                    match (default_order, order_directive.needs_to_be_reverse_order) {
                        (SortOrder::Ascending, true) => vec![first.descend(), second_column.descend()],
                        (SortOrder::Descending, true) => vec![first.ascend(), second_column.descend()],
                        (SortOrder::Ascending, false) => vec![first.ascend(), second_column.ascend()],
                        (SortOrder::Descending, false) => vec![first.descend(), second_column.ascend()],
                    }
                } else {
                    match (default_order, order_directive.needs_to_be_reverse_order) {
                        (SortOrder::Ascending, true) => vec![second_column.descend()],
                        (SortOrder::Descending, true) => vec![second_column.ascend()],
                        (SortOrder::Ascending, false) => vec![second_column.ascend()],
                        (SortOrder::Descending, false) => vec![second_column.descend()],
                    }
                }
            }
            None if order_directive.needs_implicit_id_ordering && order_directive.needs_to_be_reverse_order => {
                vec![second_column.descend()]
            }
            None if order_directive.needs_implicit_id_ordering && !order_directive.needs_to_be_reverse_order => {
                vec![second_column.ascend()]
            }
            None => vec![],
        }
    }
}
