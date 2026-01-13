class CylManagerError(Exception):
    """Base exception for CylManager"""
    pass

class ServiceError(CylManagerError):
    """Raised when a service operation fails"""
    pass

class ConfigError(CylManagerError):
    """Raised when configuration is invalid"""
    pass

class SystemRequirementError(CylManagerError):
    """Raised when system requirements are not met"""
    pass
