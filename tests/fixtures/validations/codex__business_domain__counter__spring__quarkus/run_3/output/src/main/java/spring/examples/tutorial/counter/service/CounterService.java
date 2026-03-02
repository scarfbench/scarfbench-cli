package spring.examples.tutorial.counter.service;

import jakarta.enterprise.context.ApplicationScoped;

@ApplicationScoped
public class CounterService {
    private int hits = 1;

    public int getHits() {
        return hits++;
    }
}
