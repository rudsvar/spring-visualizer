package com.example.demo.service;

import org.springframework.context.annotation.Bean;
import org.springframework.stereotype.Service;

import com.example.demo.MyBean;

@Service
public class ConstructorInjection {

    @SuppressWarnings("unused")
    private final ConstructorInjected constructorInjected;

    public ConstructorInjection(ConstructorInjected constructorInjected) {
        this.constructorInjected = constructorInjected;
    }

    @Bean
    public ConstructorInjected constructorInjected(MyBean myBean) {
        return new ConstructorInjected(myBean);
    }
}
