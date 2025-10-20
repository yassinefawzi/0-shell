pub fn Clearaw() {
    std::process::Command::new("clear")
          .status()
          .unwrap();
}
