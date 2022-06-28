package data;

@SpringBootApplication
@Import({ B.class, C.class })
@ComponentScan("data.scanned_dir")
class A {
    @Bean
    MyBean bean() { ... }
}
