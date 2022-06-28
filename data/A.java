package data;

@SpringBootApplication
@Import({ B.class, C.class })
@ComponentScan("scanned_dir")
class A {
    @Bean
    MyBean bean() { ... }
}
