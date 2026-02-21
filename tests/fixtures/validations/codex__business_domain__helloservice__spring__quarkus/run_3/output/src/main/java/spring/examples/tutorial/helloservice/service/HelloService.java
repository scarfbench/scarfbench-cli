package spring.examples.tutorial.helloservice.service;

import jakarta.enterprise.context.ApplicationScoped;

@ApplicationScoped
public class HelloService {

    private final String message = "Hello, ";

    public String sayHello(String name) {
        return message + name + ".";
    }
}
