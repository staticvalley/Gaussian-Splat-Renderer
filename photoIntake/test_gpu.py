import torch

x = torch.randn(5000, 5000, device="cuda")
y = x @ x
print(y.shape)
print(torch.cuda.get_device_name(0))