# No Code Machine Learning

Suppose we have this sample network:

<p align="center">
  <img alt="Sample network" src="docs/net.png">
  
  <p align="center">
    <sup>$X$...inputs</sup>
</p>
</p>

<!-- definition of the activation -->
```math
a^{(l)}_j = f(z^{(l)}_j)
```

Where $f$ is the activation function, e.g., sigmoid:

<!-- example of activation with the sigmoid activation function -->
```math
a^{(l)}_j = \sigma(z^{(l)}_j)
```

<!-- definition of z -->
```math
z^{(l)}_j = \sum_{k = 0}^{n_l - 1} (a^{(l - 1)}_k w^{(l)}_{jk}) + b^{(l)}_j
```
<p align="center">
  <sup>$n_l$...number of inputs in the layer $l$</sup>
</p>

<p align="center">
  <sup>$l$...any layer in the network</sup>
</p>

## All formulas for all a and z values

### $L = 3$
<!-- formula for a^{(3)}_0 -->
```math
a^{(3)}_0 = f(z^{(3)}_0)
```

<!-- formula for z^{(3)}_0 -->
```math
z^{(3)}_0 = {
  a^{(2)}_0 w^{(3)}_{00} + 
  a^{(2)}_1 w^{(3)}_{01} + 
  b^{(3)}_0
}
```

<!-- formula for a^{(3)}_1 -->
```math
a^{(3)}_1 = f(z^{(3)}_1)
```

<!-- formula for z^{(3)}_1 -->
```math
z^{(3)}_1 = {
  a^{(2)}_0 w^{(3)}_{10} + 
  a^{(2)}_1 w^{(3)}_{11} + 
  b^{(3)}_1
}
```


### $L = 2$
<!-- formula for a^{(2)}_0 -->
```math
a^{(2)}_0 = f(z^{(2)}_0)
```

<!-- formula for z^{(2)}_0 -->
```math
z^{(2)}_0 = {
  X_0 w^{(2)}_{00} + 
  X_1 w^{(2)}_{01} + 
  b^{(2)}_0
}
```

<!-- formula for a^{(2)}_1 -->
```math
a^{(2)}_1 = f(z^{(2)}_1)
```

<!-- formula for z^{(L)}_1 -->
```math
z^{(2)}_1 = {
  X_0 w^{(2)}_{10} + 
  X_1 w^{(2)}_{11} + 
  b^{(2)}_1
}
```

## Derivatives of cost functions

Suppose a universal cost function, $C$.

<!-- partial derivative of C with respect to w^{(L)}_{jk} -->
```math
\frac{\partial C}{\partial w^{(L)}_{jk}} = {
  \frac{\partial C}{\partial a^{(L)}_j}
  \frac{\partial a^{(L)}_j}{\partial z^{(L)}_j}
  \frac{\partial z^{(L)}_j}{\partial w^{(L)}_{jk}}
}
```

<!-- partial derivative of C with respect to b^{(L)}_j -->
```math
\frac{\partial C}{\partial b^{(L)}_j} = {
  \frac{\partial C}{\partial a^{(L)}_j}
  \frac{\partial a^{(L)}_j}{\partial z^{(L)}_j}
  \frac{\partial z^{(L)}_j}{\partial b^{(L)}_j}
} 
```

<!-- partial derivative of C with respect to a^{(L - 1)}_k -->
```math
\frac{\partial C}{\partial a^{(L - 1)}_k} = {
  \frac{\partial C}{\partial a^{(L)}_j}
  \frac{\partial a^{(L)}_j}{\partial z^{(L)}_j}
  \frac{\partial z^{(L)}_j}{\partial a^{(L - 1)}_k}
}
```

TODO: check and explain for earlier layers

### Mean Squared Error (MSE)

<!-- the definition of mse -->
```math
C = MSE = \sum_{j = 0}^{n_L-1}(a^{(L)}_j - y_j)^2
```

<!-- the derivative of mse -->
```math
\frac{\partial C}{\partial a^{(L)}_j} = 2(a^{(L)}_j - y_j)
```
