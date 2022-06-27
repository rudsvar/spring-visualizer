@Import({ B.class, C.class })
@ComponentScan("scanned-dir")
class A {
    @Bean
    MyBean bean() { ... }
}
