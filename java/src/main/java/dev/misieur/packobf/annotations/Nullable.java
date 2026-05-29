package dev.misieur.packobf.annotations;

import java.lang.annotation.*;

@Retention(RetentionPolicy.SOURCE)
@Target({ElementType.TYPE_USE, ElementType.METHOD, ElementType.FIELD, ElementType.PARAMETER})
@Documented
public @interface Nullable {
    String value() default "";
}
