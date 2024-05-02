// @generated automatically by Diesel CLI.

diesel::table! {
    employees (id) {
        id -> Int4,
        name -> Text,
        email -> Text,
        password -> Text,
        phone_number -> Nullable<Text>,
        department -> Nullable<Text>,
        role -> Text,
        availability -> Nullable<Json>,
    }
}

diesel::table! {
    schedules (id) {
        id -> Int4,
        employee_id -> Int4,
        data -> Date,
        shift_id -> Int4,
        note -> Nullable<Text>,
    }
}

diesel::table! {
    shift_changes (id) {
        id -> Int4,
        scheduler_id -> Int4,
        reason -> Nullable<Text>,
        status -> Nullable<Text>,
    }
}

diesel::table! {
    shifts (id) {
        id -> Int4,
        name -> Text,
        start_time -> Int4,
        end_time -> Int4,
        duration -> Nullable<Int4>,
        minium_attendences -> Nullable<Int4>,
    }
}

diesel::joinable!(schedules -> employees (employee_id));
diesel::joinable!(schedules -> shifts (shift_id));
diesel::joinable!(shift_changes -> schedules (scheduler_id));

diesel::allow_tables_to_appear_in_same_query!(
    employees,
    schedules,
    shift_changes,
    shifts,
);
