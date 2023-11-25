package com.example.demo.service;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class ConstructorInjection {

    @SuppressWarnings("unused")
    private final ConstructorInjected constructorInjected;

    // Redundant @Autowired-annotations
    @Autowired
    public ConstructorInjection(@Autowired ConstructorInjected constructorInjected) {
        this.constructorInjected = constructorInjected;
    }
}
