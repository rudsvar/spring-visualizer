package com.example.demo;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import(DaoConfig.class)
@ComponentScan({ "com.example.demo.service" })
public class ServiceConfig {
    @Bean
    public MyBean myBean() {
        return new MyBean();
    }
}
