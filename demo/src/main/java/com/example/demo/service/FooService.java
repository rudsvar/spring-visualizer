package com.example.demo.service;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import com.example.demo.repository.FooRepository;

@Service
public class FooService {
    @Autowired
    FooRepository fooRepository;
}
