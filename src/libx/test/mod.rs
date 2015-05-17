use super::*

#[test]
fn test_open(){
    let display = open_display(None);
    assert!(display.is_some());
}
