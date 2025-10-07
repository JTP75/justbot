# Rayleigh-Plesset Equation

The primary equation for bubble cavitation dynamics is the **Rayleigh-Plesset equation**, which describes the growth and collapse of a spherical bubble in an incompressible liquid:

$$R\frac{d^2R}{dt^2} + \frac{3}{2}\left(\frac{dR}{dt}\right)^2 = \frac{1}{\rho}\left[p_B(t) - p_\infty(t) - \frac{2\sigma}{R} - \frac{4\mu}{R}\frac{dR}{dt}\right]$$

Where:
- **R** = bubble radius
- **t** = time
- **ρ** = liquid density
- **p_B** = pressure inside the bubble
- **p_∞** = pressure in the liquid far from the bubble
- **σ** = surface tension
- **μ** = liquid dynamic viscosity

## Key Terms:

1. **Left side**: Inertial terms (bubble acceleration)
2. **Right side pressure terms**:
   - p_B - p_∞: Pressure difference driving bubble motion
   - 2σ/R: Surface tension (opposes expansion)
   - 4μ(dR/dt)/R: Viscous damping

## Simplified Versions:

- **Rayleigh equation**: Neglects surface tension and viscosity (inviscid, zero surface tension)
- **Minnaert frequency**: For small oscillations about equilibrium

This equation is fundamental in understanding acoustic cavitation, sonochemistry, ultrasound cleaning, and cavitation damage in pumps and propellers.