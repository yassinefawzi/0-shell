pub fn clearaw() {
    std::process::Command::new("clear")
          .status()
          .unwrap();
}
