package spring.examples.tutorial.standalone.service;

import jakarta.enterprise.context.ApplicationScoped;

@ApplicationScoped
public class StandaloneService {

    private static final String message = "Greetings!";

    public String returnMessage() {
        return message;
    }

}
