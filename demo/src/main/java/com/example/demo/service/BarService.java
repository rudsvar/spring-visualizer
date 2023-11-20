package com.example.demo.service;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Bean;
import org.springframework.stereotype.Service;

import com.example.demo.MyBean;

@Service
public class BarService {
    @Autowired
    MyBean myBean;

    @Bean
    public ConstructorInjected constructorInjected(ConstructorInjected constructorInjected) {
        return new ConstructorInjected();
    }
}
