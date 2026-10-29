//! The notification server: owns org.freedesktop.Notifications on the session bus, so every app's
//! notifications come to the island. It only speaks D-Bus; the notifications section keeps them.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gio::prelude::*;
use gtk4::{gio, glib};

use super::notice::Notice;

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";
const CAPABILITIES: [&str; 6] = ["actions", "body", "body-markup", "icon-static", "persistence", "x-canonical-private-synchronous"];
const SERVER_INFORMATION: (&str, &str, &str, &str) = ("brasa-island", "Brasa", "1.0", "1.2");

pub const EXPIRED: u32 = 1;
pub const DISMISSED: u32 = 2;
pub const CLOSED_BY_APP: u32 = 3;

const INTERFACE_XML: &str = r#"
<node>
  <interface name="org.freedesktop.Notifications">
    <method name="Notify">
      <arg direction="in" type="s"/><arg direction="in" type="u"/><arg direction="in" type="s"/>
      <arg direction="in" type="s"/><arg direction="in" type="s"/><arg direction="in" type="as"/>
      <arg direction="in" type="a{sv}"/><arg direction="in" type="i"/>
      <arg direction="out" type="u"/>
    </method>
    <method name="CloseNotification"><arg direction="in" type="u"/></method>
    <method name="GetCapabilities"><arg direction="out" type="as"/></method>
    <method name="GetServerInformation">
      <arg direction="out" type="s"/><arg direction="out" type="s"/>
      <arg direction="out" type="s"/><arg direction="out" type="s"/>
    </method>
    <signal name="NotificationClosed"><arg type="u"/><arg type="u"/></signal>
    <signal name="ActionInvoked"><arg type="u"/><arg type="s"/></signal>
  </interface>
</node>"#;

struct Handlers {
    on_notify: Box<dyn Fn(Notice) -> u32>,
    on_take_back: Box<dyn Fn(u32)>,
}

pub struct Server {
    connection: RefCell<Option<gio::DBusConnection>>,
}

impl Server {
    /// on_notify gives the id of the notice it is handed; on_take_back hears of an app closing its own.
    /// Both run on the GTK thread and must answer at once, since the app waits for the reply.
    pub fn start(on_notify: impl Fn(Notice) -> u32 + 'static, on_take_back: impl Fn(u32) + 'static) -> Rc<Server> {
        let server = Rc::new(Server { connection: RefCell::new(None) });
        let handlers = Rc::new(Handlers { on_notify: Box::new(on_notify), on_take_back: Box::new(on_take_back) });
        let waiting = Rc::clone(&server);
        glib::spawn_future_local(async move {
            match gio::bus_get_future(gio::BusType::Session).await {
                Ok(connection) => waiting.serve(connection, handlers),
                Err(error) => eprintln!("brasa-islands: no session bus for notifications: {error}"),
            }
        });
        server
    }

    pub fn closed(&self, notice_id: u32, reason: u32) {
        self.emit("NotificationClosed", &(notice_id, reason).to_variant());
    }

    pub fn invoked(&self, notice_id: u32, action_key: &str) {
        self.emit("ActionInvoked", &(notice_id, action_key).to_variant());
    }

    fn emit(&self, signal: &str, parameters: &glib::Variant) {
        let connection = self.connection.borrow();
        let Some(connection) = connection.as_ref() else { return };
        let _ = connection.emit_signal(None, PATH, NAME, signal, Some(parameters));
    }

    fn serve(&self, connection: gio::DBusConnection, handlers: Rc<Handlers>) {
        let Some(interface) = interface_info() else { return };
        let registered = connection
            .register_object(PATH, &interface)
            .method_call(move |_connection, _sender, _path, _interface, method, arguments, invocation| {
                answer(&handlers, method, &arguments, invocation);
            })
            .build();
        if let Err(error) = registered {
            eprintln!("brasa-islands: cannot serve notifications: {error}");
            return;
        }
        gio::bus_own_name_on_connection(&connection, NAME, gio::BusNameOwnerFlags::REPLACE, |_, _| {}, |_, _| {});
        *self.connection.borrow_mut() = Some(connection);
    }
}

fn interface_info() -> Option<gio::DBusInterfaceInfo> {
    let node = gio::DBusNodeInfo::for_xml(INTERFACE_XML).ok()?;
    node.interfaces().first().cloned()
}

fn answer(handlers: &Handlers, method: &str, arguments: &glib::Variant, invocation: gio::DBusMethodInvocation) {
    match method {
        "Notify" => {
            let id = (handlers.on_notify)(Notice::from_notify(arguments));
            invocation.return_value(Some(&(id,).to_variant()));
        }
        "CloseNotification" => {
            let id = arguments.child_value(0).get::<u32>().unwrap_or(0);
            (handlers.on_take_back)(id);
            invocation.return_value(None);
        }
        "GetCapabilities" => invocation.return_value(Some(&(CAPABILITIES.to_vec(),).to_variant())),
        "GetServerInformation" => invocation.return_value(Some(&SERVER_INFORMATION.to_variant())),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interface_describes_its_methods_and_signals() {
        let interface = interface_info().unwrap();
        for method in ["Notify", "CloseNotification", "GetCapabilities", "GetServerInformation"] {
            assert!(interface.lookup_method(method).is_some(), "{method}");
        }
        for signal in ["NotificationClosed", "ActionInvoked"] {
            assert!(interface.lookup_signal(signal).is_some(), "{signal}");
        }
    }
}
