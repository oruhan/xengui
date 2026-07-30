#[component]
fn Page() -> impl Widget {
    let count = use_state(0i32);

    fn increment() {
        count.set(count + 1)
    };

    rsx! {
        <View>
            <Label style={ color: "#ffffff" }>Count: {count.get()}</Label>
            <Button onClick={increment}>Increment</Button>
        </View>
    }
}