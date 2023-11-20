package com.example.demo.service;

import org.springframework.stereotype.Service;

@Service
public class ConstructorInjection {

    @SuppressWarnings("unused")
    private final ConstructorInjected constructorInjected;

    public ConstructorInjection(ConstructorInjected constructorInjected) {
        this.constructorInjected = constructorInjected;
    }
}
