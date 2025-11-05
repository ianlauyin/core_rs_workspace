use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveTime;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use framework::json;
use serde::Deserialize;
use serde::Serialize;

fn criterion_benchmark(c: &mut Criterion) {
    let json = std::fs::read_to_string("test.json").unwrap();
    c.bench_function("json", |b| {
        b.iter(|| {
            let _: TestJSON = json::from_json(&json).unwrap();
        })
    });
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestJSON {
    #[serde(rename = "planning")]
    pub planning: Planning,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Planning {
    #[serde(rename = "hdr_id")]
    pub hdr_id: String,

    #[serde(rename = "is_test_hdr")]
    pub is_test: bool,

    #[serde(rename = "code")]
    pub code: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "status")]
    pub status: HDRStatusView,

    #[serde(rename = "time_zone_id")]
    pub time_zone_id: String,

    #[serde(rename = "categories")]
    pub categories: Vec<HDRCategory>,

    #[serde(rename = "close_time_for_today")]
    pub close_time_for_today: Option<DateTime<FixedOffset>>,

    #[serde(rename = "original_opening_time_for_today")]
    pub original_opening_time_for_today: Option<DateTime<FixedOffset>>,

    #[serde(rename = "opening_time_for_today")]
    pub opening_time_for_today: Option<DateTime<FixedOffset>>,

    #[serde(rename = "opening_time")]
    pub opening_time: Option<DateTime<FixedOffset>>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "contact_number")]
    pub contact_number: String,

    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "latitude")]
    pub latitude: f64,

    #[serde(rename = "longitude")]
    pub longitude: f64,

    #[serde(rename = "address_line1")]
    pub address_line1: String,

    #[serde(rename = "address_2")]
    pub address_2: Option<String>,

    #[serde(rename = "city")]
    pub city: String,

    #[serde(rename = "state_code")]
    pub state_code: String,

    #[serde(rename = "county")]
    pub county: Option<String>,

    #[serde(rename = "zip_code")]
    pub zip_code: Option<String>,

    #[serde(rename = "logo_image_key")]
    pub logo_image_key: String,

    #[serde(rename = "detail_image_key")]
    pub detail_image_key: String,

    #[serde(rename = "coming_soon_image_key")]
    pub coming_soon_image_key: String,

    #[serde(rename = "pickup_instructions")]
    pub pickup_instructions: Option<String>,

    #[serde(rename = "customer_pickup_instructions")]
    pub customer_pickup_instructions: Option<String>,

    #[serde(rename = "location_details")]
    pub location_details: Option<String>,

    #[serde(rename = "location_type", default)]
    pub location_type: LocationType,

    #[serde(rename = "google_review_url")]
    pub google_review_url: Option<String>,

    #[serde(rename = "yelp_review_url")]
    pub yelp_review_url: Option<String>,

    #[serde(rename = "delivery_zone_id")]
    pub delivery_zone_id: Option<String>,

    #[serde(rename = "created_time")]
    pub created_time: DateTime<FixedOffset>,

    #[serde(rename = "hdr_operating_hours")]
    pub hdr_operating_hours: Vec<OperatingHour>,

    #[serde(rename = "walk_in_operating_hours")]
    pub walk_in_operating_hours: Vec<OperatingHour>,

    #[serde(rename = "restaurants")]
    pub restaurants: Vec<Restaurant>,

    #[serde(rename = "scheduled_order_configuration", default)]
    pub scheduled_order_configuration: ScheduledOrderConfiguration,

    #[serde(rename = "wonder_spot_configuration", default)]
    pub wonder_spot_configuration: WonderSpotConfiguration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FulfillmentTypeView {
    DELIVERY,
    #[serde(rename = "PICK_UP")]
    PickUp,
    #[serde(rename = "IN_STORE")]
    InStore,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum LocationType {
    WALMART,
    #[default]
    WONDER,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RestaurantPlanningStatusView {
    OPEN,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum RestaurantStatusView {
    ACTIVE,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum HDRCloseReasonView {
    OTHER,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum RestaurantStatusChangedByView {
    OTHER,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum DayOfWeekView {
    MONDAY,
    TUESDAY,
    WEDNESDAY,
    THURSDAY,
    FRIDAY,
    SATURDAY,
    SUNDAY,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HDRStatusView {
    OPEN,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Restaurant {
    #[serde(rename = "hdr_restaurant_id")]
    pub hdr_restaurant_id: String,

    #[serde(rename = "restaurant_id")]
    pub restaurant_id: String,

    #[serde(rename = "restaurant_name")]
    pub restaurant_name: String,

    #[serde(rename = "today_postponed_time")]
    pub today_postponed_time: Option<DateTime<FixedOffset>>,

    #[serde(rename = "today_original_opening_time")]
    pub today_original_opening_time: Option<DateTime<FixedOffset>>,

    #[serde(rename = "status")]
    pub status: RestaurantPlanningStatusView,

    #[serde(rename = "restaurant_status")]
    pub restaurant_status: RestaurantStatusView,

    #[serde(rename = "restaurant_close_reason")]
    pub restaurant_close_reason: Option<HDRCloseReasonView>,

    #[serde(rename = "status_changed_by")]
    pub status_changed_by: Option<RestaurantStatusChangedByView>,

    #[serde(rename = "fulfillment_types")]
    pub fulfillment_types: Vec<FulfillmentTypeView>,

    #[serde(rename = "service_time")]
    pub service_time: Option<ServiceTime>,

    #[serde(rename = "next_service_times")]
    pub next_service_times: Vec<ServiceTime>,

    #[serde(rename = "operating_hours")]
    pub operating_hours: Vec<OperatingHour>,

    #[serde(rename = "scheduled_time_slots")]
    pub scheduled_time_slots: Option<Vec<ScheduledTimeSlot>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceTime {
    #[serde(rename = "start_time")]
    pub start_time: DateTime<FixedOffset>,

    #[serde(rename = "end_time")]
    pub end_time: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperatingHour {
    #[serde(rename = "day_of_week")]
    pub day_of_week: DayOfWeekView,

    #[serde(rename = "service_hours")]
    pub service_hours: Vec<ServiceHour>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceHour {
    #[serde(rename = "start_time_est")]
    pub start_time_est: Option<NaiveTime>,

    #[serde(rename = "end_time_est")]
    pub end_time_est: Option<NaiveTime>,

    #[serde(rename = "start_time")]
    pub start_time: NaiveTime,

    #[serde(rename = "end_time")]
    pub end_time: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledTimeSlot {
    #[serde(rename = "day_of_week")]
    pub day_of_week: DayOfWeekView,

    #[serde(rename = "time_slots")]
    pub time_slots: Vec<TimeSlot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSlot {
    #[serde(rename = "start_time")]
    pub start_time: NaiveTime,

    #[serde(rename = "end_time")]
    pub end_time: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
#[derive(Default)]
pub struct ScheduledOrderConfiguration {
    #[serde(rename = "enabled")]
    pub enabled: bool,

    #[serde(rename = "from_time")]
    pub from_time: String,

    #[serde(rename = "to_time")]
    pub to_time: String,

    #[serde(rename = "cut_off_in_minutes")]
    pub cut_off_in_minutes: i32,

    #[serde(rename = "window_length_in_minutes")]
    pub window_length_in_minutes: i32,

    #[serde(rename = "release_buffer_in_minutes")]
    pub release_buffer_in_minutes: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
#[derive(Default)]
pub struct WonderSpotConfiguration {
    #[serde(rename = "partners")]
    pub partners: Vec<Partner>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Partner {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "time_slots")]
    pub time_slots: Vec<PartnerTimeSlot>,

    #[serde(rename = "cutoff_time")]
    pub cutoff_time: Option<NaiveTime>,

    #[serde(rename = "delivery_time_from")]
    pub delivery_time_from: Option<NaiveTime>,

    #[serde(rename = "delivery_time_to")]
    pub delivery_time_to: Option<NaiveTime>,

    #[serde(rename = "capacity")]
    pub capacity: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartnerTimeSlot {
    #[serde(rename = "delivery_time_from")]
    pub delivery_time_from: NaiveTime,

    #[serde(rename = "delivery_time_to")]
    pub delivery_time_to: NaiveTime,

    #[serde(rename = "cutoff_time")]
    pub cutoff_time: NaiveTime,

    #[serde(rename = "capacity")]
    pub capacity: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HDRCategory {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "properties")]
    pub properties: Vec<HDRProperty>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HDRProperty {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
