"""WeRender-ML - Zero-Config Distributed ML Training."""

__version__ = "0.1.0"

from werender_ml.coordinator import MLCoordinator
from werender_ml.worker import MLWorker

__all__ = ["MLCoordinator", "MLWorker"]
