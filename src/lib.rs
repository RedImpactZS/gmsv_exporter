#![feature(thread_local)]
#![allow(non_snake_case,unsafe_op_in_unsafe_fn)]

use gmod::lua::State;
use lazy_static::lazy_static;
use prometheus::{core::{AtomicF64, GenericCounterVec, Collector, GenericGaugeVec}, default_registry, register_counter_vec, register_gauge_vec};
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Mutex};

#[macro_use] extern crate gmod;

lazy_static! {
    static ref GLOBAL_GAUGES: Mutex<HashMap<String,(String,GenericGaugeVec<AtomicF64>)>> = Mutex::new(HashMap::new());
    static ref GLOBAL_COUNTERS: Mutex<HashMap<String,(String,GenericCounterVec<AtomicF64>)>> = Mutex::new(HashMap::new());
}

#[lua_function]
unsafe fn RegisterGauge(lua: State) -> i32 {
    let group_name = lua.check_string(1).into_owned();
    let metric_name = lua.check_string(2).into_owned();
    let description = lua.check_string(3).into_owned();
    let labels: Vec<String> = (4..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {
        let mut gauges_guard = GLOBAL_GAUGES.lock()?;
        let gauge = register_gauge_vec!(&metric_name,&description,labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?;
        
        if gauges_guard.contains_key(&metric_name) {
            return Err("Metric with same name is already registered".into());
        } else {
            let _ = gauges_guard.insert(metric_name, (group_name,gauge));
        }

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})

}

#[lua_function]
unsafe fn SetGauge(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();
    let value = match lua.is_none_or_nil(2) {
        true => 0.0,
        false => lua.check_number(2),
    };
    let labels: Vec<String> = (3..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {

        let mut gauges_guard = GLOBAL_GAUGES.lock()?;
        let gauge = gauges_guard.get_mut(&metric_name).ok_or("Gauge not found")?;
        gauge.1.get_metric_with_label_values(labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?.set(value);

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn RemoveGauge(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();
    let labels: Vec<String> = (2..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {
        let mut gauges_guard = GLOBAL_GAUGES.lock()?;
        let gauge = gauges_guard.get_mut(&metric_name).ok_or("Gauge not found")?;
        gauge.1.remove_label_values(labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?;

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn UnregisterGauge(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();

    (|| -> Result<(), Box<dyn Error>> {
        let registry = default_registry();
        let mut gauges_guard = GLOBAL_GAUGES.lock()?;
        let gauge = gauges_guard.remove(&metric_name).ok_or("Gauge not found")?;
        registry.unregister(Box::new(gauge.1) as Box<dyn Collector>)?;

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn RegisterCounter(lua: State) -> i32 {
    let group_name = lua.check_string(1).into_owned();
    let metric_name = lua.check_string(2).into_owned();
    let description = lua.check_string(3).into_owned();
    let labels: Vec<String> = (4..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {
        let mut counters_guard = GLOBAL_COUNTERS.lock()?;
        let counter = register_counter_vec!(&metric_name,&description,labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?;
        
        if counters_guard.contains_key(&metric_name) {
            return Err("Metric with same name is already registered".into());
        } else {
            let _ = counters_guard.insert(metric_name, (group_name,counter));
        }

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})

}

#[lua_function]
unsafe fn IncCounter(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();
    let labels: Vec<String> = (2..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {
        let mut counters_guard = GLOBAL_COUNTERS.lock()?;
        let counter = counters_guard.get_mut(&metric_name).ok_or("Counter not found")?;
        counter.1.get_metric_with_label_values(labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?.inc();

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn IncCounterBy(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();
    let value = match lua.is_none_or_nil(2) {
        true => 1.0,
        false => lua.check_number(2),
    };
    let labels: Vec<String> = (3..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {
        let mut counters_guard = GLOBAL_COUNTERS.lock()?;
        let counter = counters_guard.get_mut(&metric_name).ok_or("Counter not found")?;
        counter.1.get_metric_with_label_values(labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?.inc_by(value);

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn ResetCounter(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();
    let labels: Vec<String> = (2..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {
        let mut counters_guard = GLOBAL_COUNTERS.lock()?;
        let counter = counters_guard.get_mut(&metric_name).ok_or("Counter not found")?;
        counter.1.get_metric_with_label_values(labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?.reset();

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn RemoveCounter(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();
    let labels: Vec<String> = (2..10).map_while(|i| (!lua.is_none_or_nil(i)).then(|| lua.check_string(i).into_owned())).collect();

    (|| -> Result<(), Box<dyn Error>> {
        let mut counters_guard = GLOBAL_COUNTERS.lock()?;
        let counter = counters_guard.get_mut(&metric_name).ok_or("Counter not found")?;
        counter.1.remove_label_values(labels.iter().map(String::as_str).collect::<Vec<&str>>().as_slice())?;

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn UnregisterCounter(lua: State) -> i32 {
    let metric_name = lua.check_string(1).into_owned();

    (|| -> Result<(), Box<dyn Error>> {
        let registry = default_registry();
        let mut counters_guard = GLOBAL_COUNTERS.lock()?;
        let counter = counters_guard.remove(&metric_name).ok_or("Counter not found")?;
        registry.unregister(Box::new(counter.1) as Box<dyn Collector>)?;

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn Start(lua: State) -> i32 {
    let addr_raw = lua.check_string(1).into_owned();
    (|| -> Result<(), Box<dyn Error>> {
        let addr: SocketAddr = addr_raw.parse()?;
        prometheus_exporter::start(addr)?;

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[lua_function]
unsafe fn UnregisterGroup(lua: State) -> i32 {
    let group_name = lua.check_string(1).into_owned();

    (|| -> Result<(), Box<dyn Error>> {
        let registry = default_registry();

        let mut gauges_guard = GLOBAL_GAUGES.lock()?;
        gauges_guard
            .extract_if(|_,x| &x.0 == &group_name)
            .map(|x| registry.unregister(Box::new(x.1.1) as Box<dyn Collector>))
            .all(|x| x.is_ok())
            .then_some(true)
            .ok_or("Can't unregister gauges")?;

        let mut counters_guard = GLOBAL_COUNTERS.lock()?;
        counters_guard
            .extract_if(|_,x| &x.0 == &group_name)
            .map(|x| registry.unregister(Box::new(x.1.1) as Box<dyn Collector>))
            .all(|x| x.is_ok())
            .then_some(true)
            .ok_or("Can't unregister counters")?;

        Ok(())
    })().err().map(|x| {lua.push_string(&x.to_string()); 1}).unwrap_or_else(|| {lua.push_boolean(true); 1})
}

#[gmod13_open]
unsafe fn gmod13_open(lua: State) -> i32 {
    macro_rules! export_lua_function {
        ($name:ident) => {
            // _G.environ.$name
            lua.push_function($name);
            lua.set_field(-2, concat!(stringify!($name), "\0").as_ptr() as *const i8);
        };
        ($func:ident, $name:literal) => {
            // _G.environ.$name
            lua.push_function($func);
            lua.set_field(-2, lua_string!($name));
        }
    }

    lua.create_table(0, 12);
    export_lua_function!(RegisterGauge);
    export_lua_function!(SetGauge);
    export_lua_function!(RemoveGauge);
    export_lua_function!(UnregisterGauge);

    export_lua_function!(RegisterCounter);
    export_lua_function!(IncCounter);
    export_lua_function!(IncCounterBy);
    export_lua_function!(ResetCounter);
    export_lua_function!(RemoveCounter);
    export_lua_function!(UnregisterCounter);

    export_lua_function!(Start);
    export_lua_function!(UnregisterGroup);
    lua.set_global(lua_string!("exporter"));

    println!("exporter.gmod13_open");
    0
}

#[gmod13_close]
unsafe fn gmod13_close(_lua: State) -> i32 {

    println!("exporter.gmod13_close");
    0
}
