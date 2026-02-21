package spring.examples.tutorial.converter.service;

import java.math.BigDecimal;
import java.math.RoundingMode;
import jakarta.enterprise.context.ApplicationScoped;

@ApplicationScoped
public class ConverterService {
    private final BigDecimal yenRate = new BigDecimal("104.34");
    private final BigDecimal euroRate = new BigDecimal("0.007");

    public BigDecimal dollarToYen(BigDecimal dollars) {
        BigDecimal result = dollars.multiply(yenRate);
        return result.setScale(2, RoundingMode.UP);
    }

    public BigDecimal yenToEuro(BigDecimal yen) {
        BigDecimal result = yen.multiply(euroRate);
        return result.setScale(2, RoundingMode.UP);
    }
}
